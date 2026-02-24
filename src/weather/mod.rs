pub mod conditions;
pub mod humidity;
pub mod mapper;
pub mod temperature;
pub mod visibility;
pub mod wind;

pub use conditions::Condition;
pub use humidity::{Humidity, calculate_humidity};
pub use mapper::WeatherReport;
pub use temperature::{Temperature, calculate_temperature};
pub use visibility::{Visibility, calculate_visibility};
pub use wind::{Wind, WindDirection, calculate_wind};
