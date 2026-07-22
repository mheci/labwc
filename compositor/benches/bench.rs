use criterion::{black_box, criterion_group, criterion_main, Criterion};
use labwc_core::{Edge, Rect};
use labwc_workspace::WorkspaceManager;

fn bench_rect(c: &mut Criterion) {
    let r1 = Rect::new(0, 0, 800, 600);
    let r2 = Rect::new(100, 100, 400, 300);
    c.bench_function("rect_intersects", |b| {
        b.iter(|| r1.intersects(black_box(&r2)))
    });
    c.bench_function("rect_contains", |b| b.iter(|| r1.contains(black_box(&r2))));
}

fn bench_edge(c: &mut Criterion) {
    c.bench_function("edge_parse", |b| {
        b.iter(|| Edge::parse(black_box("topleft")))
    });
    c.bench_function("edge_invert", |b| b.iter(|| Edge::TOP_LEFT.invert()));
}

fn bench_workspace(c: &mut Criterion) {
    let mut mgr = WorkspaceManager::new();
    c.bench_function("workspace_switch", |b| {
        b.iter(|| mgr.switch_to(black_box("3")))
    });
}

criterion_group!(benches, bench_rect, bench_edge, bench_workspace);
criterion_main!(benches);
