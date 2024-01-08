pub struct Cloud {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub t: u8,
}

pub const CLOUD_ACCEL_DECAY: f32 = 0.75;
pub const CLOUD_DURATION: u8 = 18;
pub const CLOUD_CAP: u8 = 20;

impl Cloud {
    pub fn update_clouds(clouds: &mut Vec<Cloud>) {
        
        for cloud in &mut clouds.iter_mut() {
            cloud.x += cloud.vx;
            cloud.y += cloud.vy;
            cloud.vx *= CLOUD_ACCEL_DECAY;
            cloud.vy *= CLOUD_ACCEL_DECAY;
            cloud.t += 1;
        }
        fn x(cloud: &Cloud) -> bool { 
            cloud.t <= CLOUD_DURATION
        }
        clouds.retain(x);
    }

    pub fn try_push_cloud(clouds: &mut Vec<Cloud>, x: f32, y: f32, vx: f32, vy: f32) {
        if (clouds.len() as u8) < CLOUD_CAP {
            clouds.push(Cloud{
                x, y, vx, vy, t: 0
            });
        } 
    }
}