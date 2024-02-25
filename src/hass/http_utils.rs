pub(super) trait Downgrade {
    type Output;
    fn downgrade(self) -> Self::Output;
}
macro_rules! rewrite_method {
    ($from:tt, $frompath:path => $topath:path: $($meth:ident),* $(,)?) => {
        match $from {
            $(
                <$frompath>::$meth => <$topath>::$meth,
            )*
            x => <$topath>::from_bytes(x.as_str().as_bytes()).expect("source method is invalid as dest method"),
        }
    };
}

impl Downgrade for http::Method {
    type Output = reqwest::Method;

    fn downgrade(self) -> Self::Output {
        rewrite_method! {
            self, Self => reqwest::Method:
            CONNECT,
            DELETE,
            GET,
            HEAD,
            OPTIONS,
            PATCH,
            POST,
            PUT,
            TRACE,
        }
    }
}
