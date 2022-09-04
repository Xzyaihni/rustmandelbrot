use std::env;
use std::process;
use std::str::FromStr;
use std::fmt;

use image;

struct Image
{
    width: u32,
    height: u32,
    data: Vec<u8>
}

struct Config
{
    x: f64,
    y: f64,
    zoom: f64,
    iterations: u128,
    width: u32,
    height: u32,
    mult: f64,
    outside_color: [u8; 3],
    inside_color: [u8; 3],
    second_color: [u8; 3],
    filename: String
}

impl Config
{
    pub fn parse(args: impl Iterator<Item=String>) -> Result<Self, String>
    {
        let mut x: f64 = -0.75;
        let mut y: f64 = 0.0;

        let mut zoom: f64 = 3.0;
        let mut iterations: u128 = 100;

        let mut width: u32 = 1024;
        let mut height: u32 = 1024;

        let mut mult: f64 = 25.0;

        let mut outside_color: [u8; 3] = [0, 0, 0];
        let mut inside_color: [u8; 3] = [255, 0, 0];
        let mut second_color: [u8; 3] = [255, 0, 255];

        let mut filename: String = "output.png".to_string();

        let mut args = args.skip(1);
        while let Some(arg) = args.next()
        {
            match arg.as_str()
            {
                "-x" => x = parse_arg(args.next())?,
                "-y" => y = parse_arg(args.next())?,
                "-z" => zoom = parse_arg(args.next())?,
                "-i" => iterations = parse_arg(args.next())?,
                "-W" => width = parse_arg(args.next())?,
                "-H" => height = parse_arg(args.next())?,
                "-m" => mult = parse_arg(args.next())?,
                "--outside" => outside_color = parse_color(args.next())?,
                "--inside" => inside_color = parse_color(args.next())?,
                "--second" => second_color = parse_color(args.next())?,
                "-o" => filename = args.next().ok_or("no filename")?,
                _ => return Err(format!("unrecongnized argument: {arg}"))
            }
        }

        Ok(Config{x, y, zoom, iterations, width, height, mult,
            outside_color, inside_color, second_color, filename})
    }
}

fn help_message() -> !
{
    eprintln!("usage: {} [args]\n",
        env::args().next().expect("first always exists"));
    eprintln!("args:");
    eprintln!("    -x    x position (default -0.75)");
    eprintln!("    -y    y position (default 0)");
    eprintln!("    -z    zoom (default 3)");
    eprintln!("    -i    iterations (default 100)");
    eprintln!("    -W    image width (default 1024)");
    eprintln!("    -H    image height (default 1024)");
    eprintln!("    -m    frequency of inner color switches (default 25)");
    eprintln!("    --outside    image height (default 0,0,0)");
    eprintln!("    --inside    image height (default 255,0,0)");
    eprintln!("    --second    image height (default 255,0,255)");
    eprintln!("    -o    output filename (default output.png)");
    process::exit(1);
}

fn main()
{
    let config = Config::parse(env::args()).unwrap_or_else(|err|
    {
        eprintln!("cant parse args: {err}");
        help_message();
    });

    let mandelbrot = mandelbrot(&config);

    image::save_buffer(config.filename, &mandelbrot.data,
        mandelbrot.width, mandelbrot.height, image::ColorType::Rgb8)
        .unwrap_or_else(|err|
        {
            eprintln!("could not save image: {err}");
            process::exit(1);
        });
}

fn mandelbrot(config: &Config) -> Image
{
    let offset = config.zoom/2.0;

    let mut data = Vec::new();
    for y in 0..config.height
    {
        for x in 0..config.width
        {
            let x = config.x-offset + config.zoom*(x as f64/config.width as f64);
            let y = config.y-offset + config.zoom*(y as f64/config.height as f64 );

            data.extend(mandel_pixel(config, x, y).iter().cloned());
        }
    }

    Image{data, width: config.width, height: config.height}
}

fn mandel_pixel(config: &Config, x: f64, y: f64) -> [u8; 3]
{
    let (inside, distance) = pixel_distance(config.iterations, x, y);

    let fraction =
    {
        let current = (distance*config.mult).sin();
        if current > 1.0
        {
            1.0
        } else
        {
            current.abs()
        }
    };

    let inside_color = lerp(config.inside_color, config.second_color, fraction);

    if inside
    {
        inside_color
    } else
    {
        lerp(config.outside_color, inside_color, distance)
    }
}

fn pixel_distance(iterations: u128, x: f64, y: f64) -> (bool, f64)
{
    let (mut z_r, mut z_i) = (0.0, 0.0);

    let distance = |v0, v1| v0*v0+v1*v1;

    for i in 0..iterations
    {
        let temp_z = z_r*z_r + x - z_i*z_i;
        z_i = 2.0*z_r*z_i + y;
        z_r = temp_z;

        if distance(z_r, z_i) > 4.0
        {
            let fraction = (i as f64)/(iterations as f64);
            return (false, fraction);
        }
    }

    (true, distance(z_r, z_i))
}

fn lerp(c0: [u8; 3], c1: [u8; 3], amount: f64) -> [u8; 3]
{
    let v_lerp = |n|
    {
        (c0[n] as i16 + ((c1[n] as i16 - c0[n] as i16) as f64 * amount) as i16) as u8
    };

    let r = v_lerp(0);
    let g = v_lerp(1);
    let b = v_lerp(2);

    [r, g, b]
}

fn parse_color(arg: Option<String>) -> Result<[u8; 3], String>
{
    let arg = arg.ok_or("no argument supplied")?;

    let mut out: [u8; 3] = [0, 0, 0];
    for (index, color) in arg.split(',').enumerate()
    {
        if index>=out.len()
        {
            return Err("not enough color values".to_string());
        }

        out[index] = color.trim().parse().map_err(|err| format!("cant parse {color}: {err}"))?;
    }

    Ok(out)
}

fn parse_arg<T>(arg: Option<String>) -> Result<T, String>
where
    T: FromStr,
    <T as FromStr>::Err: fmt::Display
{
    let text = arg.ok_or("no argument supplied")?;

    text.trim().parse().map_err(|err| format!("{err}"))
}