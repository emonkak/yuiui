use crate::geometrics::PhysicalSize;

#[derive(Debug, Clone)]
pub struct WindowClose;

#[derive(Debug, Clone)]
pub struct WindowResize(pub PhysicalSize);
