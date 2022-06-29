macro_rules! rb {
    ($a:expr)=>{
        $a.as_ref().borrow()
    };
}

macro_rules! rbm {
    ($a:expr)=>{
        $a.as_ref().borrow_mut()
    };
}

macro_rules! timed_println {
    ($($arg:tt)*)=>{
        let duration = crate::EPOCH.elapsed();
        let seconds = duration.as_secs() % 60;
        let minutes = (duration.as_secs() / 60) % 60;
        let hours = (duration.as_secs() / 60) / 60;
        let handle = std::thread::current();
        print!("[{:0>2}:{:0>2}:{:0>2}] [{}]\t", hours, minutes, seconds, handle.name().unwrap_or("<>"));
        println!($($arg)*);
    };
}

pub(crate) use rb;
pub(crate) use rbm;
pub(crate) use timed_println;