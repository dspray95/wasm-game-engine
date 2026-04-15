pub struct LaserModelId(pub usize);

pub struct FireCooldown {
    pub last_fired: web_time::Instant,
}
