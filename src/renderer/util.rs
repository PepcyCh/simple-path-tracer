pub struct ImageRange {
    pub from: u32,
    pub to: u32,
}

pub fn create_image_ranges(num_thread: u32, height: u32) -> Vec<ImageRange> {
    let height_per_cpu = height / num_thread;
    let mut ranges = Vec::with_capacity(num_thread as usize);
    for t in 0..num_thread {
        let from = t * height_per_cpu;
        let to = if t + 1 == num_thread {
            height
        } else {
            (t + 1) * height_per_cpu
        };
        ranges.push(ImageRange { from, to });
    }
    ranges
}

pub fn render_prograss_bar(width: u32, height: u32) -> indicatif::ProgressBar {
    let progress_bar = indicatif::ProgressBar::new(width as u64 * height as u64);
    progress_bar.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} (eta: {eta})")
            .progress_chars("#>-"),
    );
    progress_bar
}
