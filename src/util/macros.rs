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

pub(crate) use rb;
pub(crate) use rbm;