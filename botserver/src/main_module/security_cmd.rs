#[cfg(feature = "security")]
use super::cli_locale;

pub async fn handle_security_command(args: &[String]) {
    #[cfg(feature = "security")]
    {
        if args.get(1).map(|s| s.as_str()) == Some("security") {
            let subcommand = args.get(2).map(|s| s.as_str()).unwrap_or("status");
            match subcommand {
                "fix" => {
                    if args.get(3).map(|s| s.as_str()) == Some("--bootstrap") {
                        crate::security::protection::print_bootstrap_instructions();
                        std::process::exit(0);
                    }
                    let report = crate::security::protection::run_security_fix().await;
                    println!("{}", cli_locale::t("cli-security-fix-title"));
                    println!(
                        "{}",
                        cli_locale::t_args(
                            "cli-security-line",
                            &[
                                ("name", &cli_locale::t("cli-security-firewall")),
                                ("state", &status_word(report.firewall.ok)),
                                ("output", report.firewall.output.trim()),
                            ],
                        )
                    );
                    println!(
                        "{}",
                        cli_locale::t_args(
                            "cli-security-line",
                            &[
                                ("name", &cli_locale::t("cli-security-fail2ban")),
                                ("state", &status_word(report.fail2ban.ok)),
                                ("output", report.fail2ban.output.trim()),
                            ],
                        )
                    );
                    println!(
                        "{}",
                        cli_locale::t_args(
                            "cli-security-line",
                            &[
                                ("name", &cli_locale::t("cli-security-caddy")),
                                ("state", &status_word(report.caddy.ok)),
                                ("output", report.caddy.output.trim()),
                            ],
                        )
                    );
                    println!(
                        "{}",
                        cli_locale::t_args(
                            "cli-security-overall",
                            &[(
                                "state",
                                &cli_locale::t(if report.success {
                                    "cli-security-success"
                                } else {
                                    "cli-security-partial"
                                }),
                            )],
                        )
                    );
                    std::process::exit(if report.success { 0 } else { 1 });
                }
                "status" => {
                    let report = crate::security::protection::run_security_status().await;
                    println!("{}", cli_locale::t("cli-security-status-title"));
                    println!(
                        "{}",
                        cli_locale::t_args(
                            "cli-security-status-line",
                            &[
                                ("name", &cli_locale::t("cli-security-firewall")),
                                ("output", report.firewall.output.trim()),
                            ],
                        )
                    );
                    println!(
                        "{}",
                        cli_locale::t_args(
                            "cli-security-status-line",
                            &[
                                ("name", &cli_locale::t("cli-security-fail2ban")),
                                ("output", report.fail2ban.output.trim()),
                            ],
                        )
                    );
                    println!(
                        "{}",
                        cli_locale::t_args(
                            "cli-security-status-line",
                            &[
                                ("name", &cli_locale::t("cli-security-caddy")),
                                ("output", report.caddy.output.trim()),
                            ],
                        )
                    );
                    std::process::exit(0);
                }
                _ => {
                    eprintln!("{}", cli_locale::t("cli-security-usage"));
                    std::process::exit(1);
                }
            }
        }
    }
    #[cfg(not(feature = "security"))]
    let _ = args;
}

/// `OK` / `FAIL` are operator-facing verdicts, not log text.
#[cfg(feature = "security")]
fn status_word(ok: bool) -> String {
    cli_locale::t(if ok {
        "cli-security-ok"
    } else {
        "cli-security-fail"
    })
}
