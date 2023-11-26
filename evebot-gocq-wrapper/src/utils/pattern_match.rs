#[macro_export]
macro_rules! static_match {
    {$val: expr => $($pat: expr),+ $(,)? $(=> $default: expr)?} => {
        match $val {
            $($pat => $pat),+
            $(_ => $default)?
        }
    };
}

#[macro_export]
macro_rules! get_content {
    ($val: expr) => {
        match $val {
            Ok(_v) => _v.get_content().await,
            Err(_e) => Err(_e),
        }
    };
}
