use std::error::Error;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::thread;

use gdrr_2bp::COST_COMPARATOR;
use gdrr_2bp::io::html_export::generate_solution;
use gdrr_2bp::io::json_format::JsonInstance;
use gdrr_2bp::io::parser;
use gdrr_2bp::optimization::config::Config;
use gdrr_2bp::optimization::gdrr::GDRR;
use gdrr_2bp::optimization::sol_collectors::global_sol_collector::GlobalSolCollector;
use gdrr_2bp::optimization::sol_collectors::local_sol_collector::LocalSolCollector;
use gdrr_2bp::timed_println;
use serde::de::DeserializeOwned;

fn read_json<T: DeserializeOwned>(path: &Path) -> io::Result<T> {
    let file = File::open(path).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("could not open {}: {error}", path.display()),
        )
    })?;

    serde_json::from_reader(BufReader::new(file)).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("could not parse {}: {error}", path.display()),
        )
    })
}

fn create_output(path: &Path) -> io::Result<BufWriter<File>> {
    File::create(path).map(BufWriter::new).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("could not create {}: {error}", path.display()),
        )
    })
}

fn solution_paths(input: &Path, output_dir: &Path) -> io::Result<(PathBuf, PathBuf)> {
    let stem = input.file_stem().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("input path {} has no file name", input.display()),
        )
    })?;
    let mut json_name = stem.to_os_string();
    json_name.push("_solution.json");
    let mut html_name = stem.to_os_string();
    html_name.push("_solution.html");
    Ok((output_dir.join(json_name), output_dir.join(html_name)))
}

fn prepare_output(input: &Path, output_dir: &Path) -> io::Result<(PathBuf, PathBuf)> {
    fs::create_dir_all(output_dir).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!(
                "could not create output directory {}: {error}",
                output_dir.display()
            ),
        )
    })?;
    solution_paths(input, output_dir)
}

pub fn run(
    input_path: PathBuf,
    config_path: PathBuf,
    output_dir: Option<PathBuf>,
) -> Result<(), Box<dyn Error>> {
    let mut json_instance: JsonInstance = read_json(&input_path)?;
    let config: Config = read_json(&config_path)?;
    config.validate().map_err(|message| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid config {}: {message}", config_path.display()),
        )
    })?;
    let output_paths = output_dir
        .as_deref()
        .map(|output_dir| prepare_output(&input_path, output_dir))
        .transpose()?;

    timed_println!("Config file loaded: {}", serde_json::to_string(&config)?);

    let instance = parser::generate_instance(&mut json_instance, &config);
    timed_println!(
        "Starting optimization of {} parts of {} different types for {} seconds",
        instance.total_part_qty(),
        instance.parts().len(),
        config.max_run_time.unwrap_or(usize::MAX)
    );
    timed_println!("Press Ctrl+C to terminate manually");

    let instance = Arc::new(instance);
    let config = Arc::new(config);
    let mut gdrr_thread_handlers = Vec::new();
    let mut tx_syncs = Vec::new();
    let (tx_solution_report, rx_solution_report) = channel();

    for i in 0..config.n_threads {
        let instance_thread = instance.clone();
        let config_thread = config.clone();
        let thread_name = format!("T{i}");
        let (tx_sync, rx_sync) = channel();
        let tx_solution_report_thread = tx_solution_report.clone();
        tx_syncs.push(tx_sync);

        let handle = thread::Builder::new().name(thread_name).spawn(move || {
            let local_sol_collector = LocalSolCollector::new(
                instance_thread.clone(),
                rx_sync,
                tx_solution_report_thread,
                COST_COMPARATOR,
            );
            let mut gdrr = GDRR::new(&instance_thread, &config_thread, local_sol_collector);
            gdrr.lahc();
        })?;
        gdrr_thread_handlers.push(handle);
    }

    let mut global_sol_collector = GlobalSolCollector::new(
        instance,
        config,
        tx_syncs,
        rx_solution_report,
        COST_COMPARATOR,
    );
    global_sol_collector.monitor(gdrr_thread_handlers);

    let solution = global_sol_collector
        .best_complete_solution()
        .or_else(|| global_sol_collector.best_incomplete_solution());
    let Some(solution) = solution else {
        timed_println!("No solution available");
        return Ok(());
    };
    if let Some((json_path, html_path)) = output_paths {
        let json_solution = parser::generate_json_solution(&json_instance, solution, &config_path);

        let mut writer = create_output(&json_path)?;
        serde_json::to_writer_pretty(&mut writer, &json_solution).map_err(|error| {
            io::Error::other(format!("could not write {}: {error}", json_path.display()))
        })?;
        writer.flush()?;
        timed_println!("JSON solution written to {}", json_path.display());

        let mut writer = create_output(&html_path)?;
        writer.write_all(generate_solution(&json_solution).as_bytes())?;
        writer.flush()?;
        timed_println!("HTML solution written to {}", html_path.display());
    }

    Ok(())
}
