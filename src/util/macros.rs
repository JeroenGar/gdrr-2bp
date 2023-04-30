
#[macro_export]
macro_rules! timed_println {
    ($($arg:tt)*)=>{
        let duration = crate::EPOCH.elapsed();
        let seconds = duration.as_secs() % 60;
        let minutes = (duration.as_secs() / 60) % 60;
        let hours = (duration.as_secs() / 60) / 60;
        print!("[{:0>2}:{:0>2}:{:0>2}]\t", hours, minutes, seconds);
        println!($($arg)*);
    };
}

#[macro_export]
macro_rules! timed_thread_println {
    ($($arg:tt)*)=>{
        let duration = crate::EPOCH.elapsed();
        let seconds = duration.as_secs() % 60;
        let minutes = (duration.as_secs() / 60) % 60;
        let hours = (duration.as_secs() / 60) / 60;
        let handle = std::thread::current();
        print!("[{:0>2}:{:0>2}:{:0>2}]\t<{}>\t", hours, minutes, seconds, handle.name().unwrap_or("<>"));
        println!($($arg)*);
    };
}