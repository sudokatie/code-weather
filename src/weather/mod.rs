pub mod conditions;
pub mod humidity;
pub mod mapper;
pub mod temperature;
pub mod visibility;
pub mod wind;

pub use conditions::Condition;
pub use humidity::{calculate_humidity, Humidity};
pub use mapper::WeatherReport;
pub use temperature::{calculate_temperature, Temperature};
pub use visibility::{calculate_visibility, Visibility};
pub use wind::{calculate_wind, Wind, WindDirection};
