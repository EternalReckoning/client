pub fn lerp(
    start: &nalgebra::Point3<f32>,
    stop: &nalgebra::Point3<f32>,
    progress: f32
) -> nalgebra::Point3<f32>
{
    let mut interp_vec = stop - start;
    interp_vec *= progress;

    start + interp_vec
}