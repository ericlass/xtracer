mod linear;

use linear::Vector4F;

fn main() {
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
}