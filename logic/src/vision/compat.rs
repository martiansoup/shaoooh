use opencv::core::ToInputArray;
use opencv::core::ToOutputArray;

#[cfg(all(opencv4_0_0, opencv4_11_0))]
pub fn cvt_color(
    src: &impl ToInputArray,
    dst: &mut impl ToOutputArray,
    code: i32,
    dst_cn: i32,
) -> Result<(), opencv::Error> {
    opencv::imgproc::cvt_color(
        src,
        dst,
        code,
        dst_cn,
        opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT,
    )
}

#[cfg(all(opencv4_0_0, not(opencv4_11_0)))]
pub fn cvt_color(
    src: &impl ToInputArray,
    dst: &mut impl ToOutputArray,
    code: i32,
    dst_cn: i32,
) -> Result<(), opencv::Error> {
    opencv::imgproc::cvt_color(src, dst, code, dst_cn)
}
