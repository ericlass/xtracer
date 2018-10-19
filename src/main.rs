extern crate time;
extern crate image;

mod linear;

use linear::Vector4F;
use linear::Vertex4F;

fn main() {
    let cam_pos = Vector4F {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0
    };

    let img_w = 1024;
    let img_h = 576;

    let img_plane_dist = 1.0;

    let img_ratio = img_w as f64 / img_h as f64;
    let img_plane_w = img_plane_dist / 2.0;
    let img_plane_h = img_plane_w / img_ratio;

    let img_plane_l = cam_pos.x - (img_plane_w / 2.0);
    let img_plane_b = cam_pos.y - (img_plane_h / 2.0);

    let img_pix_inc_h = img_plane_w / img_w as f64;
    let img_pix_inc_v = img_plane_h / img_h as f64;

    let sp_c = Vector4F {
        x: 0.0,
        y: 0.0,
        z: 5.0,
        w: 1.0
    };

    let sp_r = 0.5;

    let mut img = image::RgbImage::new(img_w, img_h);

    let start = time::precise_time_ns();
    for ix in 0..img_w {
        for iy in 0..img_h {
            let px = Vector4F {
                x: img_plane_l + (ix as f64 * img_pix_inc_h),
                y: img_plane_b + (iy as f64 * img_pix_inc_v),
                z: img_plane_dist,
                w: 0.0
            };

            let ray_dir = &px - &cam_pos;

            //let intersects = linear::ray_intersects_sphere(&cam_pos, &ray_dir, &sp_c, sp_r);
            //let mut pixel: image::Rgb<u8>;
            //if intersects {
            //    pixel = image::Rgb([255 as u8, 0 as u8, 0 as u8]);
            //}
            //else {
            //    pixel = image::Rgb([64 as u8, 64 as u8, 64 as u8]);
            //}
            //img.put_pixel(ix, iy, pixel);

            let intersects = linear::intersect_ray_sphere(&cam_pos, &ray_dir, &sp_c, sp_r, 1000.0);
            let mut pixel: image::Rgb<u8>;
            
            match intersects {
                Some(inter) => {
                    let normal = inter.normal;
                    pixel = image::Rgb([convert(normal.x), convert(normal.y), convert(normal.z)]);
                },
                _ => {
                    pixel = image::Rgb([64 as u8, 64 as u8, 64 as u8]);
                }
            };

            img.put_pixel(ix, iy, pixel);
            
        }
    }
    let end = time::precise_time_ns();
    let duration_ns = end as f64 - start as f64;
    let duration = duration_ns / 1000.0;
    println!("Render time: {}", duration);

    img.save("render.png").unwrap();
}

fn convert(v: f64) -> u8 {
    let mut result = v;
    if result < 0.0 {
        result = result * -1.0;
    }

    if result > 1.0 {
        result = 1.0;
    }

    result = result * 255.0;

    result as u8
}

fn generic_tests() {
        let vec = Vector4F{x: 5.0, y:3.0, z: 0.0, w: 1.0};
    let vec2 = Vector4F{x: 2.0, y:8.0, z: 1.5, w: 1.0};
    
    println!("{}", vec);
    println!("{}", vec.len());
    println!("{}", vec.sqr_len());

    println!("{}", vec2);
    let vec3 = Vector4F::add(&vec, &vec2);
    let vec4 = Vector4F::sub(&vec, &vec2);
    println!("{}", vec3);
    println!("{}", vec4);

    println!("{}", vec3 == vec4);

    let vec5 = vec3.normalize();
    let vec6 = vec4.normalize();
    println!("{}", vec5);

    println!("{}", Vector4F::dot(&vec3, &vec4));
    println!("{}", Vector4F::dot(&vec5, &vec6));

    let vec_a = Vector4F{x: 4.0, y:0.0, z: 0.0, w: 1.0};
    let vec_b = Vector4F{x: 3.0, y:1.0, z: 1.0, w: 1.0};
    let ps = Vector4F::project_scalar(&vec_b, &vec_a);
    println!("Scalar Projection: {}", ps);
    let vec_p = Vector4F::project(&vec_b, &vec_a);
    println!("Projection: {}", vec_p);

    let vecx = Vector4F{x: 1.0, y:0.0, z: 0.0, w: 1.0};
    let vecz = Vector4F{x: 0.0, y:0.0, z: -1.0, w: 1.0};
    let vecc = Vector4F::cross(&vecx, &vecz);
    println!("Cross: {}", vecc);

    let veci = Vector4F{x: 1.0, y:-1.0, z: 0.0, w: 1.0};
    let vecn = Vector4F{x: 0.0, y:1.0, z: 0.0, w: 1.0}.normalize();
    let vecr = Vector4F::reflect(&veci, &vecn);
    println!("Reflected: {}", vecr);

    let vecr = Vector4F::refract(&veci, &vecn, 1.333);
    println!("Refracted: {}", vecr);

    println!("");
    println!("=== SPHERE ===");
    test_sphere();

    println!("");
    println!("=== TRIANGLE ===");
    test_triangle();

    println!("");
    println!("=== BENCHMARK ===");
    benchmark_sphere();
    benchmark_triangle();
}

fn benchmark_sphere() {
    let count = 1000000;

    let rorg = Vector4F{x: 0.0, y:0.0, z: 0.0, w: 1.0};
    let rdir = Vector4F{x: 0.0, y:1.0, z: 0.0, w: 1.0};

    let sc = Vector4F{x: 0.0, y:3.0, z: 0.0, w: 1.0};
    let radius = 1.3333;

    let start = time::precise_time_ns();
    for _i in 0..count {
        //linear::intersect_ray_sphere(&rorg, &rdir, &sc, radius, 100.0);
        linear::ray_intersects_sphere(&rorg, &rdir, &sc, radius);
    }    
    let end = time::precise_time_ns();

    let duration = end - start;
    let average = (duration as f64) / (count as f64);
    println!("Ray/Sphere Duration: {}ns", duration);
    println!("Ray/Sphere Average : {}ns", average);
}

fn test_sphere() {
    let rorg = Vector4F{x: 0.0, y:0.0, z: 0.0, w: 1.0};
    let rdir = Vector4F{x: 0.0, y:1.0, z: 0.0, w: 1.0};

    let sc = Vector4F{x: 0.0, y:3.0, z: 0.0, w: 1.0};
    let radius = 1.3333;

    let is = linear::intersect_ray_sphere(&rorg, &rdir, &sc, radius, 100.0).unwrap();
    println!("Pos: {}", is.pos);
    println!("Normal: {}", is.normal);
    println!("T: {}", is.ray_t);
}

fn benchmark_triangle() {
    let count = 1000000;

    let rorg = Vector4F{x: 0.0, y:0.0, z: 0.0, w: 1.0};
    let rdir = Vector4F{x: 5.1, y:0.0, z: 0.0, w: 1.0};

    let v1 = Vertex4F {
        pos: Vector4F{x: 5.0, y:1.0, z: 0.0, w: 1.0},
        normal: Vector4F{x: -1.0, y:0.0, z: 0.0, w: 1.0},
        tex: Vector4F{x: 0.0, y:0.0, z: 0.0, w: 1.0},
        color: Vector4F{x: 0.0, y:0.0, z: 0.0, w: 1.0},
    };
    let v2 = Vertex4F {
        pos: Vector4F{x: 5.0, y:-1.0, z: -1.0, w: 1.0},
        normal: Vector4F{x: -1.0, y:0.0, z: 0.0, w: 1.0},
        tex: Vector4F{x: 0.0, y:0.0, z: 0.0, w: 1.0},
        color: Vector4F{x: 0.0, y:0.0, z: 0.0, w: 1.0},
    };
    let v3 = Vertex4F {
        pos: Vector4F{x: 5.0, y:-1.0, z: 1.0, w: 1.0},
        normal: Vector4F{x: -1.0, y:0.0, z: 0.0, w: 1.0},
        tex: Vector4F{x: 0.0, y:0.0, z: 0.0, w: 1.0},
        color: Vector4F{x: 0.0, y:0.0, z: 0.0, w: 1.0},
    };

    let start = time::precise_time_ns();
    for _i in 0..count {
        //linear::intersect_ray_triangle(&rorg, &rdir, &v1, &v2, &v3, 100.0);
        linear::ray_intersects_triangle(&rorg, &rdir, &v1, &v2, &v3);
    }    
    let end = time::precise_time_ns();

    let duration = end - start;
    let average = (duration as f64) / (count as f64);
    println!("Ray/Triangle Duration: {}ns", duration);
    println!("Ray/Triangle Average : {}ns", average);
}

fn test_triangle() {
    let rorg = Vector4F{x: 0.0, y:0.0, z: 0.0, w: 1.0};
    let rdir = Vector4F{x: 5.1, y:0.0, z: 0.0, w: 1.0};

    let v1 = Vertex4F {
        pos: Vector4F{x: 5.0, y:1.0, z: 0.0, w: 1.0},
        normal: Vector4F{x: -1.0, y:0.0, z: 0.0, w: 1.0},
        tex: Vector4F{x: 0.0, y:0.0, z: 0.0, w: 1.0},
        color: Vector4F{x: 0.0, y:0.0, z: 0.0, w: 1.0},
    };
    let v2 = Vertex4F {
        pos: Vector4F{x: 5.0, y:-1.0, z: -1.0, w: 1.0},
        normal: Vector4F{x: -1.0, y:0.0, z: 0.0, w: 1.0},
        tex: Vector4F{x: 0.0, y:0.0, z: 0.0, w: 1.0},
        color: Vector4F{x: 0.0, y:0.0, z: 0.0, w: 1.0},
    };
    let v3 = Vertex4F {
        pos: Vector4F{x: 5.0, y:-1.0, z: 1.0, w: 1.0},
        normal: Vector4F{x: -1.0, y:0.0, z: 0.0, w: 1.0},
        tex: Vector4F{x: 0.0, y:0.0, z: 0.0, w: 1.0},
        color: Vector4F{x: 0.0, y:0.0, z: 0.0, w: 1.0},
    };

    let is = linear::intersect_ray_triangle(&rorg, &rdir, &v1, &v2, &v3, 100.0).unwrap();

    println!("Pos: {}", is.pos);
    println!("Normal: {}", is.normal);
    println!("Tex: {}", is.tex);
    println!("Bary: {}", is.barycentric);
    println!("Ray T: {}", is.ray_t.sqrt());
}