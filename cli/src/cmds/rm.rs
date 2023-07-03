use crate::Output;
use anyhow::Context;

/// Remove a transaction
#[derive(clap::Parser)]
pub struct Rm {
    /// Transaction date
    date: lib::Date,

    /// Index of transaction in DATE
    index: usize,

    /// Execute the removal instead of displaying dry run changes
    #[arg(short, long)]
    yes: bool,
}

impl Rm {
    pub fn run(
        self,
        mut rl: lib::Recordlist,
        charset: lib::Charset,
        config: &lib::Config,
        fs: &lib::Fs,
    ) -> anyhow::Result<Output> {
        let iid0 = self.index.wrapping_sub(config.first_index_in_date);
        if rl.get(self.date, iid0).is_none() {
            anyhow::bail!("nonexistent transaction");
        }

        let rl_for_date = rl
            .slice_spanning_interval(lib::Interval {
                start: self.date,
                end: self.date,
            })
            .iter()
            .collect::<lib::Recordlist>();
        let lspp = move |config: &lib::tree::forview::Config,
                         r: &lib::Record,
                         iid0: usize,
                         mut leaf_string: String|
              -> String {
            if r.date() == self.date && iid0 == iid0 {
                if self.yes {
                    leaf_string.insert_str(0, config.charset.color_prefix_red);
                    leaf_string.push_str(" <- [REMOVED]");
                } else {
                    leaf_string.insert_str(0, config.charset.color_prefix_yellow);
                    leaf_string.push_str(" <- [WOULD BE REMOVED]");
                }
                leaf_string.push_str(config.charset.color_suffix);
            }
            leaf_string
        };
        let tr_config = lib::tree::forview::Config {
            charset,
            first_iid: config.first_index_in_date,
            rl: rl_for_date,
            leaf_string_postprocessor: Some(Box::new(lspp)),
        };

        if self.yes {
            rl.remove(self.date, iid0)
                .expect("record should have already been verified to exist");
            fs.write(&rl).with_context(|| {
                format!(
                    "failed to write '{}'",
                    fs.path::<lib::Recordlist>().display()
                )
            })?;
        }

        Ok(Output::TreeForView(tr_config))
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::util::testing::env;
    use crate::util::testing::Env;

    #[rstest]
    #[case(
        Rm {
            date: lib::Date::MIN,
            index: 0,
            yes: true,
        },
        "",
    )]
    #[case(
        Rm {
            date: lib::Date::MIN,
            index: 0,
            yes: false,
        },
        r#"
            {"d":"2015-03-01","c":"abc","a":111}
            {"d":"2015-03-30","c":"category","a":111}
            {"d":"2015-03-30","c":"category","a":111}
            {"d":"2015-03-30","c":"category","a":111}
            {"d":"2015-04-01","c":"category","a":111}
        "#,
    )]
    fn test_bad_index(env: Env, #[case] rm: Rm, #[case] rl: lib::Recordlist) {
        let charset = lib::Charset::default();
        let res = rm.run(rl, charset, &env.config, &env.fs);
        assert!(res.is_err());
    }

    #[rstest]
    #[case(
        Rm {
            date: lib::Date::MIN,
            index: 0,
            yes: false,
        },
        r#"
            {"d":"0000-01-01","c":"abc","a":111}
            {"d":"0000-01-01","c":"def","a":111,"n":"note"}
        "#,
        r#"
            {"d":"0000-01-01","c":"abc","a":111}
            {"d":"0000-01-01","c":"def","a":111,"n":"note"}
        "#,
        "abc <- [WOULD BE REMOVED]"
    )]
    #[case(
        Rm {
            date: lib::Date::MIN,
            index: 1,
            yes: false,
        },
        r#"
            {"d":"0000-01-01","c":"abc","a":111}
            {"d":"0000-01-01","c":"def","a":111,"n":"note"}
        "#,
        r#"
            {"d":"0000-01-01","c":"abc","a":111}
            {"d":"0000-01-01","c":"def","a":111,"n":"note"}
        "#,
        "def: note <- [WOULD BE REMOVED]"
    )]
    #[case(
        Rm {
            date: lib::Date::MIN,
            index: 1,
            yes: true,
        },
        r#"
            {"d":"0000-01-01","c":"abc","a":111}
            {"d":"0000-01-01","c":"def","a":111,"n":"note"}
        "#,
        r#"{"d":"0000-01-01","c":"abc","a":111}"#,
        "def: note <- [REMOVED]"
    )]
    fn test_specifying_yes(
        env: Env,
        #[case] rm: Rm,
        #[case] rl: lib::Recordlist,
        #[case] want_rl: lib::Recordlist,
        #[case] want_in_output: &str,
    ) {
        env.fs.write(&rl).unwrap();
        let output = rm
            .run(rl, lib::Charset::default(), &env.config, &env.fs)
            .unwrap()
            .to_string();
        assert_eq!(env.fs.read::<lib::Recordlist>().unwrap(), want_rl);
        assert!(output.contains(want_in_output));
    }
}
