//! Exact orthogonal geometry rules independent of storage, rendering, or electrical objects.
use ecad_domain::*;
fn p(x: i64, y: i64) -> DrawingPoint {
    DrawingPoint { x, y }
}
#[test]
fn explicit_pivot_rotations_and_mirrors_have_exact_document_coordinate_conventions() {
    let pivot = p(10, -20);
    let source = p(13, -16);
    for (turn, expected) in [
        (QuarterTurn::Positive90, p(6, -17)),
        (QuarterTurn::Half, p(7, -24)),
        (QuarterTurn::Negative90, p(14, -23)),
    ] {
        assert_eq!(
            DrawingTransform::Rotate { pivot, turn }.point(source),
            Ok(expected)
        );
        assert_eq!(
            DrawingTransform::Rotate { pivot, turn }.point(pivot),
            Ok(pivot)
        );
    }
    assert_eq!(
        DrawingTransform::Mirror {
            pivot,
            axis: MirrorAxis::Vertical
        }
        .point(source),
        Ok(p(7, -16))
    );
    assert_eq!(
        DrawingTransform::Mirror {
            pivot,
            axis: MirrorAxis::Horizontal
        }
        .point(source),
        Ok(p(13, -24))
    );
}
#[test]
fn quarter_turn_cycles_and_double_mirrors_restore_all_sample_coordinates_without_rounding() {
    for pivot in [p(0, 0), p(-5, 11)] {
        for source in [p(-1001, 1003), p(17, -23), pivot] {
            let rotate = DrawingTransform::Rotate {
                pivot,
                turn: QuarterTurn::Positive90,
            };
            let mut result = source;
            for _ in 0..4 {
                result = rotate.point(result).unwrap();
            }
            assert_eq!(result, source);
            let back = DrawingTransform::Rotate {
                pivot,
                turn: QuarterTurn::Negative90,
            };
            assert_eq!(back.point(rotate.point(source).unwrap()).unwrap(), source);
            for axis in [MirrorAxis::Horizontal, MirrorAxis::Vertical] {
                let mirror = DrawingTransform::Mirror { pivot, axis };
                assert_eq!(mirror.point(mirror.point(source).unwrap()).unwrap(), source);
            }
        }
    }
}
#[test]
fn full_span_offsets_are_allowed_but_unrepresentable_results_return_typed_errors() {
    let axis = DrawingTransform::Mirror {
        pivot: p(i64::MAX, 0),
        axis: MirrorAxis::Horizontal,
    };
    assert_eq!(axis.point(p(i64::MIN, 1)), Ok(p(i64::MIN, -1)));
    let overflow = DrawingTransform::Mirror {
        pivot: p(0, 0),
        axis: MirrorAxis::Vertical,
    };
    assert_eq!(
        overflow.point(p(i64::MIN, 0)),
        Err(DrawingError::CoordinateOverflow)
    );
    let rotate = DrawingTransform::Rotate {
        pivot: p(i64::MAX, i64::MAX),
        turn: QuarterTurn::Positive90,
    };
    assert_eq!(
        rotate.point(p(i64::MIN, i64::MAX)),
        Ok(p(i64::MAX, i64::MIN))
    );
    assert_eq!(
        rotate.point(p(i64::MAX, i64::MIN)),
        Err(DrawingError::CoordinateOverflow)
    );
}
