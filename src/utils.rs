macro_rules! variant {
    ($expression_in:expr, $pattern:pat => $expression_out:expr) => {
        match $expression_in {
            $pattern => $expression_out,
            variant => panic!("{:?}", variant),
        }
    };

    ($expression_in:expr, $pattern:pat => $expression_out:expr, $panic:expr) => {
        match $expression_in {
            $pattern => $expression_out,
            _ => panic!($panic),
        }
    };
}

macro_rules! dev_err {
    ($cause:expr) => {
        Error::DevError {
            line: line!(),
            file: file!(),
            cause: $cause.to_string(),
        }
    };
}
