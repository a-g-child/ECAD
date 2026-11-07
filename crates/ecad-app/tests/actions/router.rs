//! Input ownership, ordered sequences, repeat, replay, and context capture regressions.

use super::*;

#[test]
fn direct_keys_and_exclusive_scopes_use_the_same_registered_action_ids() {
    let registry = ActionRegistry::builtins();
    let session = session();
    let mut context = context(&session);
    let mut router = router(&registry);
    let KeyRoute::Invoke(invocation) = router.route(
        &registry,
        event(1, chord('t', false, false), false),
        &context,
        0,
    ) else {
        panic!("T did not invoke");
    };
    assert_eq!(invocation.action, id("drawing.label.create"));
    context.focus = ActionFocus::ProjectTree;
    let KeyRoute::Invoke(invocation) = router.route(
        &registry,
        event(2, chord('e', false, false), false),
        &context,
        1,
    ) else {
        panic!("tree Properties did not invoke");
    };
    assert_eq!(invocation.action, id("selection.properties"));
    assert!(matches!(
        router.route(
            &registry,
            event(3, chord('t', false, false), false),
            &context,
            2
        ),
        KeyRoute::Unhandled
    ));
}

#[test]
fn text_numeric_modal_ime_and_control_owned_input_do_not_start_tools_or_steal_undo() {
    let registry = ActionRegistry::builtins();
    let session = session();
    let baseline = context(&session);
    let mut router = router(&registry);
    for (index, focus) in [ActionFocus::TextInput, ActionFocus::Modal]
        .into_iter()
        .enumerate()
    {
        let mut context = baseline.clone();
        context.focus = focus;
        assert!(matches!(
            router.route(
                &registry,
                event(
                    u64::try_from(index).unwrap() + 1,
                    chord('t', false, false),
                    false
                ),
                &context,
                0
            ),
            KeyRoute::Suppressed(InputSuppression::Protected)
        ));
    }
    let mut composing = baseline.clone();
    composing.composing = true;
    assert!(matches!(
        router.route(
            &registry,
            event(3, chord('z', true, false), false),
            &composing,
            0
        ),
        KeyRoute::Suppressed(InputSuppression::Protected)
    ));
    let mut control = baseline;
    control.control_owns_key = true;
    assert!(matches!(
        router.route(
            &registry,
            event(4, chord('t', false, false), false),
            &control,
            0
        ),
        KeyRoute::Suppressed(InputSuppression::Protected)
    ));
    assert!(router.pending().is_none());
}

#[test]
fn sequences_preserve_original_anchor_and_cancel_on_target_or_focus_changes() {
    let registry = ActionRegistry::builtins();
    let session = session();
    let original = context(&session);
    let mut router = router(&registry);
    assert!(matches!(
        router.route(
            &registry,
            event(1, chord('k', true, false), false),
            &original,
            10
        ),
        KeyRoute::Pending {
            deadline_ms: 1_010,
            ..
        }
    ));
    let mut moved = original.clone();
    moved.anchor = Some(DrawingPoint { x: 999, y: 999 });
    let KeyRoute::Invoke(invocation) = router.route(
        &registry,
        event(2, chord('s', false, false), false),
        &moved,
        20,
    ) else {
        panic!("sequence failed");
    };
    assert_eq!(invocation.action, id("drawing.styles.edit"));
    assert_eq!(invocation.context.anchor, original.anchor);
    router.route(
        &registry,
        event(3, chord('k', true, false), false),
        &original,
        30,
    );
    moved
        .selection
        .insert(ObjectRef::View(ViewId::new(1).unwrap()));
    assert!(matches!(
        router.route(
            &registry,
            event(4, chord('s', false, false), false),
            &moved,
            40
        ),
        KeyRoute::Suppressed(InputSuppression::ContextChanged)
    ));
    router.route(
        &registry,
        event(5, chord('k', true, false), false),
        &original,
        50,
    );
    moved.focus = ActionFocus::TextInput;
    assert!(matches!(
        router.route(
            &registry,
            event(6, chord('s', false, false), false),
            &moved,
            60
        ),
        KeyRoute::Suppressed(InputSuppression::Protected)
    ));
    assert!(router.pending().is_none());
}

#[test]
fn timeout_escape_bad_continuation_and_clock_reversal_have_deterministic_cancellation() {
    let registry = ActionRegistry::builtins();
    let session = session();
    let mut context = context(&session);
    context.tool_active = true;
    let mut router = router(&registry);
    router.route(
        &registry,
        event(1, chord('k', true, false), false),
        &context,
        0,
    );
    assert!(!router.expire_pending(999));
    assert!(router.expire_pending(1_000));
    assert!(matches!(
        router.route(
            &registry,
            event(2, chord('s', false, false), false),
            &context,
            1_001
        ),
        KeyRoute::Unhandled
    ));
    router.route(
        &registry,
        event(3, chord('k', true, false), false),
        &context,
        1_002,
    );
    let escape = ShortcutChord {
        key: ShortcutKey::named(NamedKey::Escape),
        modifiers: ShortcutModifiers::default(),
    };
    assert!(matches!(
        router.route(&registry, event(4, escape.clone(), false), &context, 1_003),
        KeyRoute::Suppressed(InputSuppression::SequenceCancelled)
    ));
    let KeyRoute::Invoke(invocation) =
        router.route(&registry, event(5, escape, false), &context, 1_004)
    else {
        panic!("tool cancel failed");
    };
    assert_eq!(invocation.action, id("placement.cancel"));
    router.route(
        &registry,
        event(6, chord('k', true, false), false),
        &context,
        1_005,
    );
    assert!(matches!(
        router.route(
            &registry,
            event(7, chord('t', false, false), false),
            &context,
            1_006
        ),
        KeyRoute::Unhandled
    )); // Never restart a tool on a failed continuation.
    router.route(
        &registry,
        event(8, chord('k', true, false), false),
        &context,
        1_007,
    );
    assert!(matches!(
        router.route(
            &registry,
            event(9, chord('s', false, false), false),
            &context,
            1_000
        ),
        KeyRoute::Suppressed(InputSuppression::ClockReversed)
    ));
    assert!(router.pending().is_none());
}

#[test]
fn repeat_and_duplicate_delivery_do_not_create_uncontrolled_placement_or_extend_pending_input() {
    let registry = ActionRegistry::builtins();
    let session = session();
    let context = context(&session);
    let mut router = router(&registry);
    assert!(matches!(
        router.route(
            &registry,
            event(1, chord('t', false, false), true),
            &context,
            0
        ),
        KeyRoute::Suppressed(InputSuppression::Repeat)
    ));
    assert!(matches!(
        router.route(
            &registry,
            event(2, chord('z', true, false), true),
            &context,
            1
        ),
        KeyRoute::Invoke(_)
    ));
    router.route(
        &registry,
        event(3, chord('k', true, false), false),
        &context,
        2,
    );
    assert!(matches!(
        router.route(
            &registry,
            event(3, chord('k', true, false), false),
            &context,
            2
        ),
        KeyRoute::Suppressed(InputSuppression::Replayed)
    ));
    assert!(router.pending().is_some());
    assert!(matches!(
        router.route(
            &registry,
            event(4, chord('s', false, false), true),
            &context,
            500
        ),
        KeyRoute::Suppressed(InputSuppression::Repeat)
    ));
    assert!(router.expire_pending(1_002));
    router.route(
        &registry,
        event(5, chord('k', true, false), false),
        &context,
        1_003,
    );
    router.replace_keymap(
        compile_keymap(
            &registry,
            &KeymapOverrides::default(),
            ShortcutPlatform::Linux,
        )
        .unwrap(),
    );
    assert!(router.pending().is_none());
    assert!(matches!(
        router.route(
            &registry,
            event(5, chord('k', true, false), false),
            &context,
            1_003
        ),
        KeyRoute::Suppressed(InputSuppression::Replayed)
    ));
}
