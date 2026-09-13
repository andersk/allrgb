use image::{ExtendedColorType::Rgb8, save_buffer};

/** The image will be 8**SIZE px on each side. */
const SIZE: u32 = 4;
const GAIN: u32 = 8 - 2 * SIZE;

fn fractal(mut t: u32) -> (u32, u32) {
    let (mut u, mut v, mut s) = (0, 0, 2);
    while t != 0 {
        if t & 1 != 0 {
            (u, v) = (v + s, s - 1 - u);
        }
        if t & 2 != 0 {
            (u, v) = (2 * s - 1 - v, u);
        }
        if t & 4 != 0 {
            (u, v) = (u + 2 * s, v + 2 * s);
        }
        t >>= 3;
        s <<= 2;
    }
    (u, v)
}

fn main() {
    let [_, out_file] = Vec::from_iter(std::env::args()).try_into().unwrap();

    let buf = Vec::from_iter((3 << 3 * SIZE - 1..5 << 3 * SIZE - 1).flat_map(|j| {
        let (j0, j1) = fractal(j);
        (3 << 3 * SIZE - 1..5 << 3 * SIZE - 1).flat_map(move |i| {
            let (i0, i1) = fractal(i);
            let mut k = 2 * i0 + j0 + 1;
            if k & 1 << 2 * SIZE == 0 {
                k = !k;
            }
            [(!j1 << GAIN) as u8, (!i1 << GAIN) as u8, (k << GAIN) as u8]
        })
    }));

    eprintln!("Saving {out_file:?}");
    save_buffer(out_file, &buf, 1 << 3 * SIZE, 1 << 3 * SIZE, Rgb8).unwrap();
}
