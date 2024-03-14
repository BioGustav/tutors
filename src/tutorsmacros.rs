macro_rules! dbglog {
    ( $debug:expr, $command:literal, $($key:literal, $val:expr),+ ) => {
        if $debug {
            log($command, vec![$(($key, $val)),+]);
        }
    };
    ( $debug:expr, $command:literal, $val:expr ) => {
        if $debug {
            println!("{:9}: {:?}", $command, $val);
        }
    }
}

macro_rules! consts {
    ( $( $name:ident: $type:ty = $val:expr; )* ) => {
        $(
            const $name: $type = $val;
        )*
    };
    (pub $( $name:ident: $type:ty = $val:expr; )* ) => {
        $(
            pub const $name: $type = $val;
        )*
    };
}
