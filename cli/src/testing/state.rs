/// The expected or actual objects deserialized from a repo directory. Unset
/// fields correspond to nonexistent files.
#[derive(Debug, PartialEq, Eq, Default)]
pub struct State {
    config: Option<base::Config>,
    rl: Option<base::Recordlist>,
    limits: Option<base::Limits>,
}

impl State {
    /// Constructs the representation of an empty directory.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets repo's [`base::Config`].
    pub fn with_config<T>(mut self, config: T) -> Self
    where
        T: TryInto<base::Config> + std::fmt::Debug,
        <T as TryInto<base::Config>>::Error: std::fmt::Debug,
    {
        self.config = Some(config.try_into().unwrap());
        self
    }

    /// Sets repo's [`base::Recordlist`].
    pub fn with_rl<T>(mut self, rl: T) -> Self
    where
        T: TryInto<base::Recordlist> + std::fmt::Debug,
        <T as TryInto<base::Recordlist>>::Error: std::fmt::Debug,
    {
        self.rl = Some(rl.try_into().unwrap());
        self
    }

    /// Sets repo's [`base::Limits`].
    pub fn with_limits<T>(mut self, limits: T) -> Self
    where
        T: TryInto<base::Limits> + std::fmt::Debug,
        <T as TryInto<base::Limits>>::Error: std::fmt::Debug,
    {
        self.limits = Some(limits.try_into().unwrap());
        self
    }

    /// Deserializes objects from `fs`.
    pub fn from_fs(fs: &base::Fs) -> Self {
        macro_rules! read {
            ($t:ty) => {{
                let p = fs.path::<$t>();
                if p.exists() {
                    Some(fs.read::<$t>().unwrap())
                } else {
                    None
                }
            }};
        }

        Self {
            config: read!(base::Config),
            rl: read!(base::Recordlist),
            limits: read!(base::Limits),
        }
    }
}

/// Representation of a repo directory's file contents. Unset fields correspond
/// to nonexistent files.
#[derive(Default)]
pub struct StrState<'a> {
    config: Option<&'a str>,
    rl: Option<&'a str>,
    limits: Option<&'a str>,
}

impl<'a> StrState<'a> {
    /// Constructs the representation of an empty directory.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets repo's [`base::Config`] file contents.
    pub fn with_config(mut self, s: &'a str) -> Self {
        self.config = Some(s);
        self
    }

    /// Sets repo's [`base::Recordlist`] file contents.
    pub fn with_rl(mut self, s: &'a str) -> Self {
        self.rl = Some(s);
        self
    }

    /// Sets repo's [`base::Limits`] file contents.
    pub fn with_limits(mut self, s: &'a str) -> Self {
        self.limits = Some(s);
        self
    }

    /// Writes string contents verbatim to `fs`. Panics if any field is not a
    /// valid serialization of a real type.
    pub fn to_fs(&self, fs: &base::Fs) {
        fn write<T>(fs: &base::Fs, field: Option<&str>)
        where
            T: std::fmt::Debug + base::fs::Io,
            <T as std::str::FromStr>::Err: std::error::Error,
        {
            if let Some(s) = field {
                let obj = s.parse::<T>();
                assert!(obj.is_ok(), "{:?}", obj);
                std::fs::write(fs.path::<T>(), s).unwrap()
            }
        }

        write::<base::Config>(fs, self.config);
        write::<base::Recordlist>(fs, self.rl);
        write::<base::Limits>(fs, self.limits);
    }

    pub fn to_state(&self) -> State {
        let mut os = State::new();
        if let Some(s) = self.config {
            os = os.with_config(s);
        }
        if let Some(s) = self.rl {
            os = os.with_rl(s);
        }
        if let Some(s) = self.limits {
            os = os.with_limits(s);
        }
        os
    }
}
