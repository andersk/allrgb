use rand::prelude::*;
use rayon::prelude::*;
use soup::prelude::*;
use std::env;
use std::error::Error;
use std::fs::File;
use std::path::Path;

fn encode(image::Rgb([r, g, b]): &image::Rgb<u8>) -> u32 {
    (*r as u32) << 16 | (*g as u32) << 8 | *b as u32
}

fn decode(p: u32) -> image::Rgb<u8> {
    image::Rgb([(p >> 16) as u8, (p >> 8) as u8, p as u8])
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    args.next().unwrap();
    let mirror_path = args.next().unwrap();
    let out_file = args.next().unwrap();
    assert!(args.next().is_none());

    let mirror_path = Path::new(&mirror_path);
    let soup = Soup::from_reader(File::open(mirror_path.join("index.html"))?)?;
    let paths: Vec<_> = soup
        .tag("ul")
        .class("hfeed")
        .find()
        .unwrap()
        .children()
        .filter(|node| !node.get("class").unwrap().starts_with("ad "))
        .map(|node| {
            let href = node.children().next().unwrap().get("href").unwrap();
            assert!(href.starts_with("https://allrgb.com/"));
            mirror_path
                .join("images")
                .join(href["https://allrgb.com/".len()..].to_string() + ".png")
        })
        .collect();
    assert!(paths.len() >= (1 << 8) - 1);

    eprint!("Loading");
    let mut src = vec![[0u8; 2]; (1 << 8) - 1 << 24];
    src.par_chunks_mut(1 << 24)
        .zip(paths)
        .map(|(chunk, path)| {
            for (x, y, p) in image::open(path)?.to_rgb().enumerate_pixels() {
                chunk[encode(p) as usize] = [(x >> 4) as u8, (y >> 4) as u8];
            }
            eprint!(".");
            Ok(())
        })
        .collect::<image::ImageResult<()>>()?;
    eprintln!();

    let mut dst: Vec<Option<u32>> = vec![None; 1 << 24];
    let mut update = 0;
    let mut rng = StdRng::seed_from_u64(0);
    let mut colors: Vec<u32> = (0..1 << 24).collect();
    let mut next = 0;

    'outer: while next != 1 << 24 {
        if update == 0 {
            update = 500000;
            eprint!("\r\x1b[K{}", (1 << 24) - next);
        }
        update -= 1;

        colors.swap(next, rng.gen_range(next, 1 << 24));
        let mut p = colors[next];
        let n = rng.gen_range(1, 1 << 8);
        let [x, y] = src[((n - 1) << 24 | p) as usize];
        let mut x = (n & !(!0 << 4)) << 8 | x as u32;
        let mut y = (n >> 4 & !(!0 << 4)) << 8 | y as u32;
        while let Some(q) = dst[(x | y << 12) as usize].replace(p) {
            if x == 0 && y == 0 {
                colors[next] = q;
                continue 'outer;
            }
            x >>= 4;
            y >>= 4;
            p = q;
        }
        next += 1;
    }
    eprint!("\r\x1b[K");

    let mut buf = vec![0u8; 3 << 24];
    for (o, p) in buf.chunks_mut(3).zip(dst) {
        o.copy_from_slice(&decode(p.unwrap()).0);
    }
    eprintln!("Saving {:?}", out_file);
    image::save_buffer(out_file, &buf, 1 << 12, 1 << 12, image::RGB(8))?;

    Ok(())
}
