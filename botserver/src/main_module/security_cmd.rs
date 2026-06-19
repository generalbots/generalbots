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
                    println!("=== Security Fix Report ===");
                    println!("Firewall : {} — {}", if report.firewall.ok { "OK" } else { "FAIL" }, report.firewall.output.trim());
                    println!("Fail2ban : {} — {}", if report.fail2ban.ok { "OK" } else { "FAIL" }, report.fail2ban.output.trim());
                    println!("Caddy    : {} — {}", if report.caddy.ok { "OK" } else { "FAIL" }, report.caddy.output.trim());
                    println!("Overall  : {}", if report.success { "SUCCESS" } else { "PARTIAL" });
                    std::process::exit(if report.success { 0 } else { 1 });
                }            "status" => {
                    let report = crate::security::protection::run_security_status().await;
                    println!("=== Security Status ===");
                    println!("Firewall : {}", report.firewall.output.trim());
                    println!("Fail2ban : {}", report.fail2ban.output.trim());
                    println!("Caddy    : {}", report.caddy.output.trim());
                    std::process::exit(0);
                }
                _ => {
                    eprintln!("Usage: botserver security <fix|status>");
                    std::process::exit(1);
                }
            }
        }
    }
    #[cfg(not(feature = "security"))]
    let _ = args;
}
