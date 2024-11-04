


// TODO: Refactor logging library
// create panicking version
// simplify dependencies and macro invocation
// use const static functions to pre-evaluate transforms

#[macro_export]
macro_rules! __diag {
    ( $diag:expr, $colored:expr, $msg:expr ) => {
        {
            let prefix = format!("{}: ", $diag);
            let colored = format!("{}: ", $colored);

            let subsequent_indent = format!("{:1$}", " ", prefix.chars().count());
            let opts = textwrap::Options::new(textwrap::termwidth() - prefix.chars().count())
                .initial_indent("")
                .subsequent_indent(subsequent_indent.as_str());
            let msg = format!($msg);
            eprintln!("{}{}", colored, colored::Colorize::bold(textwrap::fill(msg.as_str(), opts).as_str()));
        }
    };
}

#[macro_export]
macro_rules! __sub_diag {
    ( $( $sub:ident = $sub_msg:expr );+ ) => {
        {
            $(
                let prefix = format!("   = {}: ", stringify!($sub));
                let token = colored::Colorize::color("=", colored::Color::Blue);
                let colored = format!("   {} {}: ", token, stringify!($sub));

                let subsequent_indent = format!("{:1$}", " ", prefix.chars().count());
                let opts = textwrap::Options::new(textwrap::termwidth() - prefix.chars().count())
                    .initial_indent("")
                    .subsequent_indent(subsequent_indent.as_str());
                let msg = format!($sub_msg);
                eprintln!("{}{}", colored, colored::Colorize::bold(textwrap::fill(msg.as_str(), opts).as_str()));
            )*
            eprintln!();
        }
    };
}

#[macro_export]
macro_rules! warn {
    ( $msg:expr ) => {
        {
            let diag = "warning";
            let colored = colored::Colorize::yellow(diag);
            let colored = colored::Colorize::bold(colored);
            crate::__diag!(diag, colored, $msg);
        }
    };
    ( $msg:expr; $( $rest:tt )+ ) => {
        {
            let diag = "warning";
            let colored = colored::Colorize::yellow(diag);
            let colored = colored::Colorize::bold(colored);
            crate::__diag!(diag, colored, $msg);
            crate::__sub_diag!( $($rest)* );
        }
    };
}

#[macro_export]
macro_rules! info {
    ( $msg:expr ) => {
        {
            let diag = "info";
            let colored = colored::Colorize::blue(diag);
            let colored = colored::Colorize::bold(colored);
            crate::__diag!(diag, colored, $msg);
        }
    };
    ( $msg:expr; $( $rest:tt )+ ) => {
        {
            let diag = "info";
            let colored = colored::Colorize::blue(diag);
            let colored = colored::Colorize::bold(colored);
            crate::__diag!(diag, colored, $msg);
            crate::__sub_diag!( $($rest)* );
        }
    };
}

#[macro_export]
macro_rules! error {
    ( $msg:expr ) => {
        {
            let diag = "info";
            let colored = colored::Colorize::red(diag);
            let colored = colored::Colorize::bold(colored);
            crate::__diag!(diag, colored, $msg);
        }
    };
    ( $msg:expr; $( $rest:tt )+ ) => {
        {
            let diag = "info";
            let colored = colored::Colorize::red(diag);
            let colored = colored::Colorize::bold(colored);
            crate::__diag!(diag, colored, $msg);
            crate::__sub_diag!( $($rest)* );
        }
    };
}
