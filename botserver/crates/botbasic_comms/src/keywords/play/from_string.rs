use super::*;

impl PlayOptions {
    pub fn from_string(options_str: &str) -> Self {
        let mut opts = Self {
            controls: true,
            ..Self::default()
        };

        for opt in options_str.split(',').map(|s| s.trim().to_lowercase()) {
            match opt.as_str() {
                "autoplay" => opts.autoplay = true,
                "loop" => opts.loop_content = true,
                "fullscreen" => opts.fullscreen = true,
                "muted" => opts.muted = true,
                "nocontrols" => opts.controls = false,
                "linenumbers" => opts.line_numbers = Some(true),
                _ => {
                    if let Some((key, value)) = opt.split_once('=') {
                        // Spaces around `=` are user formatting, not part of the
                        // key or the value: "page = 3" must behave like "page=3".
                        match (key.trim(), value.trim()) {
                            ("start", value) => opts.start_time = value.parse().ok(),
                            ("end", value) => opts.end_time = value.parse().ok(),
                            ("width", value) => opts.width = value.parse().ok(),
                            ("height", value) => opts.height = value.parse().ok(),
                            ("theme", value) => opts.theme = Some(value.to_string()),
                            ("slide", value) => opts.slide = value.parse().ok(),
                            ("page", value) => opts.page = value.parse().ok(),
                            ("zoom", value) => opts.zoom = value.parse().ok(),
                            ("highlight", value) => {
                                opts.highlight_lines =
                                    Some(value.split('-').filter_map(|s| s.parse().ok()).collect());
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        opts
    }
}
