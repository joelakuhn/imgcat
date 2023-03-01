use std::process::exit;
use std::cmp::min;

use argparse::{ArgumentParser, Collect, StoreOption, StoreTrue};

use terminal_size::{Width, Height, terminal_size};

use image::io::Reader as ImageReader;
use image::{GenericImageView};
use image::imageops::FilterType::{Lanczos3, Nearest, Triangle};
use image::DynamicImage;

fn ceil_half(n : u32) -> u32 {
    return n / 2 + n % 2
}

fn get_output_size(img : &DynamicImage, spec_w : Option<u32>, spec_h : Option<u32>, real_size : bool) -> (u32, u32) {
    if real_size {
        return (img.width(), ceil_half(img.height()));
    }

    let (term_w, term_h) = match terminal_size() {
        Some((Width(w), Height(h))) => (w as u32, (h * 2) as u32),
        None => (80, 60),
    };

    let ratio = (img.width() as f32) / (img.height() as f32);

    if spec_w.is_some() && spec_h.is_some() {
        return (spec_w.unwrap(), ceil_half(spec_h.unwrap()));
    }
    else if spec_w.is_some() {
        return (spec_w.unwrap(), ceil_half(((spec_w.unwrap() as f32) / ratio) as u32));
    }
    else if spec_h.is_some() {
        return (((ratio * (spec_h.unwrap() * 2) as f32) as u32) / 2, ceil_half(spec_h.unwrap()));
    }

    let bounds_w = min(term_w, img.width());
    let bounds_h = min(term_h, img.height());

    let width_based_height = ((bounds_w as f32) / ratio).floor() as u32;
    let height_based_width = (ratio * (bounds_h as f32)).floor() as u32;

    if width_based_height > bounds_h {
        return (height_based_width, ceil_half(bounds_h));
    }
    else {
        return (bounds_w, ceil_half(width_based_height));
    }
}

fn parse_optional_int(opt : Option<String>) -> Option<u32> {
    let parsed_opt = match opt {
        Some(w_arg) => match u32::from_str_radix(w_arg.as_str(), 10) {
            Ok(w) => Some(w),
            Err(_e) => {
                eprintln!("Malformed size: {}", w_arg);
                exit(1);
            }
        },
        None => None
    };
    return parsed_opt;
}

fn maybe_resize(img : DynamicImage, algo : image::imageops::FilterType, target_w : u32, target_h : u32, real_size : bool) -> DynamicImage {
    return if real_size {
        img
    }
    else if target_w == img.width() && target_h * 2 == img.height() {
        img
    }
    else {
        img.resize_exact(target_w, target_h * 2, algo)
    }
}

fn print_img(img : DynamicImage, algo : image::imageops::FilterType, target_w : u32, target_h : u32, real_size : bool) {
    let resized = maybe_resize(img, algo, target_w, target_h, real_size);

    let mut output = String::from("");
    let max_y = resized.height() - 1;

    for row in 0..target_h {
        for col in 0..target_w {
            let x = col;
            let upper_y = row * 2;
            let lower_y = row * 2 + 1;

            let upper_px = resized.get_pixel(x, upper_y);
            let lower_px = if lower_y < max_y {
                resized.get_pixel(x, lower_y)
            }
            else {
                image::Rgba([0, 0, 0, 0])
            };

            let image::Rgba([ur, ug, ub, _ua]) = upper_px;
            let image::Rgba([lr, lg, lb, _la]) = lower_px;
            output += format!("\x1b[48;2;{};{};{};38;2;{};{};{}m\u{2584}\x1b[0m", ur, ug, ub, lr, lg, lb).as_str();
        }
        output += "\n";
    }
    print!("{}", output);
}

fn main() {
    let mut positional_args : Vec<String> = vec![];
    let mut specified_width_arg : Option<String> = None;
    let mut specified_height_arg : Option<String> = None;
    let mut real_size = false;
    let mut triangle = true;
    let mut lanczos = false;
    let mut nearest = false;

    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut specified_width_arg)
            .add_option(&["-w", "--width"], StoreOption,
            "Specify width");
        ap.refer(&mut specified_height_arg)
            .add_option(&["-h", "--height"], StoreOption,
            "Specify height");
        ap.refer(&mut real_size)
            .add_option(&["-S", "--real-size"], StoreTrue,
            "Print image at real size");
        ap.refer(&mut triangle)
            .add_option(&["-t", "--triangle"], StoreTrue,
            "Use triangle algorithm (default)");
        ap.refer(&mut nearest)
            .add_option(&["-n", "--nearest"], StoreTrue,
            "Use nearest neighbor algorithm (faster, low quality)");
        ap.refer(&mut lanczos)
            .add_option(&["-l", "--lanczos"], StoreTrue,
            "Use lanczos3 algorithm (slower, high quality)");
        ap.refer(&mut positional_args)
            .add_argument("files", Collect,
            "Image files to process");
        ap.parse_args_or_exit();
    }

    let spec_w = parse_optional_int(specified_width_arg);
    let spec_h = parse_optional_int(specified_height_arg);
    let print_paths = positional_args.len() > 1;
    let algo = if nearest { Nearest } else if lanczos { Lanczos3 } else { Triangle };

    for path in positional_args {
        if print_paths {
            println!("{}", path);
        }
        if let Ok(reader) = ImageReader::open(path.as_str()) {
            if let Ok(img) = reader.decode() {
                let (target_w, target_h) = get_output_size(&img, spec_w, spec_h, real_size);

                print_img(img, algo, target_w, target_h, real_size);
            }
            else {
                eprintln!("Could not decode {}", path);
            }
        }
        else {
            eprintln!("Could not open {}", path);
        }
    }
}
