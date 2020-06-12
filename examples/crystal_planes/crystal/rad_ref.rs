#[allow(unused_imports)]
use super::{
    aligned_vector_init, Bitmap, BlockMap, DisplayWrap, MutRadSlice, RadBuffer, RadFrontend,
    RadSlice,
};
use super::{
    ffs::{self, Extent},
    util, PlanesSep,
};
use crate::math::prelude::*;
use amethyst::core::math;
use rayon::prelude::*;
use std::sync::Mutex;
use std::time::Instant;

pub struct RadBackend {
    pub emit: Vec<Vec3>,
    pub extents: Vec<Vec<ffs::Extent>>,
    pub rad_front: RadBuffer,
    pub rad_back: RadBuffer,
    pub diffuse: Vec<Vec3>,
}

fn vec_mul(v1: &Vec3, v2: &Vec3) -> Vec3 {
    Vec3::new(v1.x * v2.x, v1.y * v2.y, v1.z * v2.z)
}

impl RadBackend {
    pub fn new(extents: Vec<Vec<Extent>>) -> Self {
        let num_planes = extents.len();
        RadBackend {
            emit: vec![Vec3::new(0.0, 0.0, 0.0); num_planes],
            rad_front: RadBuffer::new(num_planes),
            rad_back: RadBuffer::new(num_planes),
            extents: extents,
            diffuse: vec![Vec3::new(1f32, 1f32, 1f32); num_planes],
        }
    }

    pub fn do_rad(&mut self, frontend: &Mutex<RadFrontend>) -> usize {
        self.do_rad_extents(frontend)
    }

    pub fn do_rad_extents(&mut self, frontend: &Mutex<RadFrontend>) -> usize {
        {
            let mut frontend = frontend.lock().expect("rad frontend lock failed");

            frontend.output = self.rad_back.clone();
            self.emit = frontend.emit.clone();
            self.diffuse = frontend.diffuse.clone();
        }
        std::mem::swap(&mut self.rad_front, &mut self.rad_back);

        let mut pint = 0;
        for (i, extents) in self.extents.iter().enumerate() {
            let mut rad_r = 0f32;
            let mut rad_g = 0f32;
            let mut rad_b = 0f32;
            let diffuse = self.diffuse[i as usize];

            let RadBuffer { r, g, b } = &self.rad_back;
            for ffs::Extent { start, ffs } in extents {
                for (j, ff) in ffs.iter().enumerate() {
                    unsafe {
                        rad_r += r.get_unchecked(j + *start as usize) * diffuse.x * *ff;
                        rad_g += g.get_unchecked(j + *start as usize) * diffuse.y * *ff;
                        rad_b += b.get_unchecked(j + *start as usize) * diffuse.z * *ff;
                    }
                }
                pint += ffs.len();
            }

            self.rad_front.r[i as usize] = self.emit[i as usize].x + rad_r;
            self.rad_front.g[i as usize] = self.emit[i as usize].y + rad_g;
            self.rad_front.b[i as usize] = self.emit[i as usize].z + rad_b;
        }
        pint
    }
}
