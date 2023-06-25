use core::{mock_platform::add_gaussian_noise, platform::Loader};
use curry_macro::curry;

#[derive(Debug)]
struct InputParams {
  vel: f64,
  pos: f64,
}

fn main() {
  let mut loader: Loader<'static, InputParams> = {
    let vel_loader = Loader::new(|| 4.0);
    let pos_loader = Loader::new(|| 5.0);
    pos_loader.map(add_gaussian_noise(0.2)).combine_load(
      vel_loader
        .map(add_gaussian_noise(0.01))
        .map(curry!(|vel, pos| InputParams { vel: vel, pos: pos })),
    )
  };
  let x1 = loader.load();
  let x2 = loader.load();
  print!("loader: {:?} \nloaded: {:?} \n {:?}", loader, x1, x2);
}
