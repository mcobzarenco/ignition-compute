#define_import_path ignition::kernel

const PI: f32 = 3.14159;

fn spiky_kernel_grad(r: vec2<f32>, h: f32) -> vec2<f32> {
    let length = length(r);
    if (length < h) {
        let grad_normalizer = 30.0 / (PI * pow(h, 5.0));
	let deviation = h - length;
	return -grad_normalizer * deviation * deviation * r / (length + 1e-5);
    } 

    return vec2<f32>(0.0, 0.0);
}

fn cubic_spline_kernel(r: vec2<f32>, h: f32) -> f32 {
    let q = length(r) / h;
    if (q < 1.0) {
        let normalizer = 40.0 / (7.0 * PI * h * h);
	if (q < 0.5) {
	    let q2 = q * q;
	    let value = 6.0 * (q2 * q - q2) + 1.0;
	    return value * normalizer;
	} else {
	    let u = 1.0 - q;
	    let value = 2.0 * u * u * u;
	    return value * normalizer;
	}
    }
    return 0.0;
}

