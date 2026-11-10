use super::*;

use crate::i18n::cli_locale::{t, t_args};

/// Default SSH host used by the `certs` helpers (kept out of the catalogue so
/// no locale file hardcodes an address).
pub(super) const DEFAULT_SSH_HOST: &str = "10.0.0.1";

pub(super) fn print_usage() {
    println!("{}", t_args("cli-usage-title", &[("version", VERSION)]));
    println!();
    println!("{}", t("cli-usage-synopsis"));
    println!();
    println!("{}", t("cli-usage-commands"));
    println!("  install <component>  {}", t("cli-usage-install"));
    println!("  remove <component>   {}", t("cli-usage-remove"));
    println!("  list                 {}", t("cli-usage-list"));
    println!("  status <component>   {}", t("cli-usage-status"));
    println!("  start                {}", t("cli-usage-start"));
    println!("  stop                 {}", t("cli-usage-stop"));
    println!("  restart              {}", t("cli-usage-restart"));
    println!("  setup-env            {}", t("cli-usage-setup-env"));
    println!("  certs <subcommand>   {}", t("cli-usage-certs"));
    println!("  vault <subcommand>   {}", t("cli-usage-vault"));
    println!("  rotate-secret <comp> {}", t("cli-usage-rotate-secret"));
    println!("                      {}", t("cli-usage-rotate-secret-list"));
    println!("  rotate-secrets --all {}", t("cli-usage-rotate-secrets"));
    println!("  version [--all]      {}", t("cli-usage-version-all"));
    println!("  --version, -v        {}", t("cli-usage-version"));
    println!("  --help, -h           {}", t("cli-usage-help"));
    println!();
    println!("{}", t("cli-usage-options"));
    println!("  --container          {}", t("cli-usage-option-container"));
    println!("  --container-only     {}", t("cli-usage-option-container-only"));
    println!("  --tenant <name>      {}", t("cli-usage-option-tenant"));
    println!("  --lang <locale>      {}", t("cli-usage-option-lang"));
    println!();
    println!("{}", t("cli-usage-certs-title"));
    println!("  certs generate       {}", t("cli-usage-certs-generate"));
    println!("  certs push           {}", t("cli-usage-certs-push"));
    println!("  certs provision      {}", t("cli-usage-certs-provision"));
    println!("  {}", t("cli-usage-options"));
    println!(
        "    --ssh-host <ip>    {}",
        t_args("cli-usage-certs-ssh-host", &[("host", DEFAULT_SSH_HOST)])
    );
    println!();
    println!("{}", t("cli-usage-security-title"));
    println!("  sudo botserver install protection   {}", t("cli-usage-security-install"));
    println!("  sudo botserver remove protection    {}", t("cli-usage-security-remove"));
    println!("  botserver status protection         {}", t("cli-usage-security-status"));
    println!();
    println!("{}", t("cli-usage-vault-title"));
    println!("  vault migrate [.env] {}", t("cli-usage-vault-migrate"));
    println!("  vault put <path> k=v {}", t("cli-usage-vault-put"));
    println!("  vault get <path>     {}", t("cli-usage-vault-get"));
    println!("  vault list           {}", t("cli-usage-vault-list"));
    println!("  vault health         {}", t("cli-usage-vault-health"));
}

pub(super) fn print_certs_usage() {
    println!("{}", t("cli-certs-title"));
    println!();
    println!("{}", t("cli-certs-synopsis"));
    println!();
    println!("{}", t("cli-certs-subcommands"));
    println!("  generate             {}", t("cli-certs-generate"));
    println!("  push                 {}", t("cli-certs-push"));
    println!("  provision            {}", t("cli-certs-provision"));
    println!();
    println!("{}", t("cli-usage-options"));
    println!(
        "  --ssh-host <ip>      {}",
        t_args("cli-certs-option-ssh-host", &[("host", DEFAULT_SSH_HOST)])
    );
    println!("  --container          {}", t("cli-certs-option-container"));
    println!("  --tenant <name>      {}", t("cli-certs-option-tenant"));
}

pub(super) fn install_protection() -> Result<()> {
    eprintln!("{}", t("cli-protection-install"));
    Ok(())
}

pub(super) fn remove_protection() -> Result<()> {
    eprintln!("{}", t("cli-protection-remove"));
    Ok(())
}

pub(super) fn verify_protection() -> bool {
    eprintln!("{}", t("cli-protection-verify"));
    false
}

pub(super) fn print_vault_usage() {
    println!("{}", t("cli-vault-title"));
    println!();
    println!("{}", t("cli-vault-synopsis"));
    println!();
    println!("{}", t("cli-certs-subcommands"));
    println!("  migrate [file]       {}", t("cli-vault-migrate-desc"));
    println!("                       {}", t("cli-vault-default-file"));
    println!();
    println!("  put <path> k=v...    {}", t("cli-vault-put-desc"));
    println!("                       {}", t("cli-vault-put-example"));
    println!();
    println!("  get <path> [key]     {}", t("cli-vault-get-desc"));
    println!("                       {}", t("cli-vault-get-example"));
    println!();
    println!("  list                 {}", t("cli-vault-list-desc"));
    println!();
    println!("  health               {}", t("cli-vault-health-desc"));
    println!();
    println!("{}", t("cli-vault-secret-paths"));
    println!("  gbo/tables           {}", t("cli-vault-path-tables"));
    println!("  gbo/drive            {}", t("cli-vault-path-drive"));
    println!("  gbo/email            {}", t("cli-vault-path-email"));
    println!("  gbo/cache            {}", t("cli-vault-path-cache"));
    println!("  gbo/directory        {}", t("cli-vault-path-directory"));
    println!("  gbo/llm              {}", t("cli-vault-path-llm"));
    println!("  gbo/encryption       {}", t("cli-vault-path-encryption"));
    println!("  gbo/stripe           {}", t("cli-vault-path-stripe"));
    println!("  gbo/vectordb         {}", t("cli-vault-path-vectordb"));
    println!();
    println!("{}", t("cli-vault-environment"));
    println!("  VAULT_ADDR           {}", t("cli-vault-env-addr"));
    println!("  VAULT_TOKEN          {}", t("cli-vault-env-token"));
}
