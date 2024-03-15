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
