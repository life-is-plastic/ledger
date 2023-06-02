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
    pub fn run<W>(
        self,
        mut stdout: W,
        mut rl: lib::Recordlist,
        charset: &lib::Charset,
        config: &lib::Config,
        fs: &lib::Fs,
    ) -> anyhow::Result<()>
    where
        W: std::io::Write,
    {
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
        let tr = lib::Tree::from(lib::tree::forview::Config {
            charset,
            first_iid: config.first_index_in_date,
            rl: &rl_for_date,
            leaf_string_postprocessor: Some(&|lspp| {
                let mut s = lspp.leaf_string;
                if lspp.r.date() == self.date && lspp.iid0 == iid0 {
                    if self.yes {
                        s.insert_str(0, charset.color_prefix_red);
                        s.push_str(" <- [REMOVED]");
                    } else {
                        s.insert_str(0, charset.color_prefix_yellow);
                        s.push_str(" <- [WOULD BE REMOVED]");
                    }
                    s.push_str(charset.color_suffix);
                }
                s
            }),
        });

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
        write!(stdout, "{}", tr)?;
        Ok(())
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
    fn test_bad_index(mut env: Env, #[case] rm: Rm, #[case] rl: lib::Recordlist) {
        let res = rm.run(
            &mut env.stdout,
            rl,
            &lib::Charset::default(),
            &env.config,
            &env.fs,
        );
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
        mut env: Env,
        #[case] rm: Rm,
        #[case] rl: lib::Recordlist,
        #[case] want_rl: lib::Recordlist,
        #[case] want_in_output: &str,
    ) {
        env.fs.write(&rl).unwrap();
        rm.run(
            &mut env.stdout,
            rl,
            &lib::Charset::default(),
            &env.config,
            &env.fs,
        )
        .unwrap();
        assert_eq!(env.fs.read::<lib::Recordlist>().unwrap(), want_rl);
        assert!(std::str::from_utf8(&env.stdout)
            .unwrap()
            .contains(want_in_output));
    }
}
