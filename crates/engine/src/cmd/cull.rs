//! Assisted culling: score photos for focus and clipping, group similar shots taken close
//! together (bursts) and mark the sharpest of each group; optionally reject blurry ones and pick
//! each group's best. Scores are kept on the photos (rules / smart albums: `sharpness`,
//! `bestOfGroup`).

use lightcraft_catalog::{Analysis, Flag, Op, PhotoId};
use serde_json::{Value, json};

use super::{CommandSpec, always, cmd};
use crate::{Result, Session};

/// Seconds between shots of the same burst at most.
const BURST_GAP: i64 = 10;
/// Signature similarity for "the same scene".
const SAME: f32 = 0.93;

fn secs(iso: &str) -> Option<i64> {
    let d = lightcraft_meta::DateTime::parse_iso(iso)?;
    Some(((d.year as i64 * 372 + d.month as i64 * 31 + d.day as i64) * 24 + d.hour as i64) * 3600 + d.minute as i64 * 60 + d.second as i64)
}

fn analyze(s: &mut Session, p: &Value) -> Result<Value> {
    let ids: Vec<PhotoId> = if p.get("ids").is_some() || s.selection.ids.len() > 1 { s.targets(p) } else { s.visible_cloned() };
    let reject_below = p.get("rejectBelow").and_then(Value::as_f64).map(|v| v as f32);
    let pick_best = p.get("pickBest").and_then(Value::as_bool).unwrap_or(false);
    // measure (thumbnail-level sources: fast, and enough for focus at 512 px)
    let mut rows: Vec<(PhotoId, Option<i64>, f32, f32, [f32; 64])> = Vec::new();
    let mut failed = Vec::new();
    for id in ids {
        match s.source_now(id, crate::media::SourceLevel::Thumb) {
            Ok(src) => {
                let t = s.catalog.photo(id).and_then(|p| p.captured.as_deref().and_then(secs));
                use lightcraft_pipeline::cull;
                rows.push((id, t, cull::sharpness(&src), cull::clipped(&src), cull::signature(&src)));
            }
            Err(e) => failed.push(json!([id.0, e])),
        }
    }
    // bursts: consecutive (by capture time) look-alike shots
    rows.sort_by_key(|r| (r.1.unwrap_or(i64::MAX), r.0));
    let mut group_of: Vec<Option<u32>> = vec![None; rows.len()];
    let mut next = 1u32;
    for i in 1..rows.len() {
        let (a, b) = (&rows[i - 1], &rows[i]);
        let close = matches!((a.1, b.1), (Some(x), Some(y)) if (y - x).abs() <= BURST_GAP);
        if close && lightcraft_pipeline::cull::similarity(&a.4, &b.4) >= SAME {
            let g = *group_of[i - 1].get_or_insert_with(|| {
                next += 1;
                next - 1
            });
            group_of[i] = Some(g);
        }
    }
    let mut best: std::collections::HashMap<u32, (usize, f32)> = Default::default();
    for (i, g) in group_of.iter().enumerate() {
        if let Some(g) = g {
            let e = best.entry(*g).or_insert((i, -1.0));
            if rows[i].2 > e.1 {
                *e = (i, rows[i].2);
            }
        }
    }
    let mut ops = Vec::new();
    let (mut rejected, mut picked) = (0, 0);
    let mut out = Vec::new();
    for (i, (id, _, sharp, clip, _)) in rows.iter().enumerate() {
        let g = group_of[i];
        let is_best = g.is_some_and(|g| best.get(&g).is_some_and(|b| b.0 == i));
        ops.push(Op::SetAnalysis { id: *id, analysis: Some(Analysis { sharpness: *sharp, clipped: *clip, group: g, best: is_best }) });
        let flag = s.catalog.photo(*id).map(|p| p.flag).unwrap_or_default();
        if reject_below.is_some_and(|t| *sharp < t) && flag != Flag::Reject {
            ops.push(Op::SetFlag { id: *id, flag: Flag::Reject });
            rejected += 1;
        } else if pick_best && is_best && flag == Flag::None {
            ops.push(Op::SetFlag { id: *id, flag: Flag::Pick });
            picked += 1;
        }
        out.push(json!({"id": id.0, "sharpness": (sharp * 10.0).round() / 10.0, "clipped": clip, "group": g, "best": is_best}));
    }
    if !ops.is_empty() {
        s.commit("Assisted Culling", Op::Batch { ops })?;
    }
    Ok(json!({"photos": out, "groups": best.len(), "rejected": rejected, "picked": picked, "failed": failed}))
}

/// Find Similar: photos that look like `id` (signatures from thumbnail-level sources, cached by
/// content), most similar first; the view is filtered to them.
fn find_similar(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "library.findSimilar";
    let id = p.get("id").and_then(Value::as_u64).map(PhotoId).or(s.active()).ok_or_else(|| super::bad(C, "no photo"))?;
    let min = p.get("similarity").and_then(Value::as_f64).unwrap_or(0.8) as f32;
    let all: Vec<PhotoId> = s.catalog.photos().filter(|q| q.in_library()).map(|q| q.id).collect();
    let sig = |s: &mut Session, id: PhotoId| -> Option<[f32; 64]> {
        let key = s.catalog.photo(id).map(|p| crate::media::content_key(p))?;
        if let Some(v) = s.signatures.get(&key) {
            return Some(*v);
        }
        let src = s.source_now(id, crate::media::SourceLevel::Thumb).ok()?;
        let v = lightcraft_pipeline::cull::signature(&src);
        s.signatures.insert(key, v);
        Some(v)
    };
    let me = sig(s, id).ok_or_else(|| super::bad(C, "can't read the photo"))?;
    let mut hits: Vec<(PhotoId, f32)> = Vec::new();
    for q in all {
        if let Some(v) = sig(s, q) {
            let sim = lightcraft_pipeline::cull::similarity(&me, &v);
            if sim >= min {
                hits.push((q, sim));
            }
        }
    }
    hits.sort_by(|a, b| b.1.total_cmp(&a.1));
    if p.get("filter").and_then(Value::as_bool).unwrap_or(true) {
        s.filter.only = hits.iter().map(|h| h.0).collect();
    }
    Ok(json!({"photos": hits.iter().map(|(i, v)| json!({"id": i.0, "similarity": (v * 1000.0).round() / 1000.0})).collect::<Vec<_>>()}))
}

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "library.findSimilar",
            "Find Similar Photos",
            ["Photo"],
            None,
            "{id?, similarity?: 0..1 (0.8), filter?: bool (true)} — photos that look like the active one (composition and tones), most similar first; filters the view to them (Clear Filters to go back) → {photos: [{id, similarity}]}",
            always,
            find_similar
        ),
        cmd!(
            "photo.analyze",
            "Assisted Culling",
            [],
            None,
            "{ids?, rejectBelow?: sharpness 0..100, pickBest?: bool} — score the selected photos (else all in view) for focus (0..100) and clipping, group look-alike shots taken within 10 s and mark the sharpest of each; optionally reject blurry photos and pick each group's best; one undo step → {photos: [{id, sharpness, clipped, group, best}], groups, rejected, picked}",
            always,
            analyze
        ),
        cmd!(
            "cull.duplicates",
            "Find Duplicate Photos",
            [],
            None,
            "{ids?, similarity?: 0..1 (0.98), rejectOthers?: bool (false)} — group near-identical photos by content signature (regardless of capture time); optionally reject every group member but its sharpest; one undo step when rejecting → {groups: [[id]], rejected}",
            always,
            duplicates
        ),
    ]
}

/// Near-duplicate detection: union-find over content signatures (RapidRAW-style
/// auto-culling). Same-scene bursts minutes apart still group; exact files
/// group first by content key. Capped at 2000 photos to bound the O(n²) pass.
fn duplicates(s: &mut Session, p: &Value) -> Result<Value> {
    use lightcraft_catalog::{Flag, Op};
    let threshold = p.get("similarity").and_then(Value::as_f64).unwrap_or(0.98) as f32;
    let threshold = if threshold.is_finite() { threshold.clamp(0.5, 1.0) } else { 0.98 };
    let reject = p.get("rejectOthers").and_then(Value::as_bool).unwrap_or(false);
    let mut ids: Vec<PhotoId> = if p.get("ids").is_some() || s.selection.ids.len() > 1 { s.targets(p) } else { s.visible_cloned() };
    ids.truncate(2000);
    // signatures (cached by content) + sharpness for the keeper.
    let mut sigs: Vec<(PhotoId, [f32; 64], f32)> = Vec::new();
    for id in ids {
        let key = s.catalog.photo(id).map(|p| crate::media::content_key(p));
        let sig = key.clone().and_then(|k| s.signatures.get(&k).copied()).or_else(|| {
            let src = s.source_now(id, crate::media::SourceLevel::Thumb).ok()?;
            let v = lightcraft_pipeline::cull::signature(&src);
            if let Some(k) = key.clone() {
                s.signatures.insert(k, v);
            }
            Some(v)
        });
        let sharp = s.source_now(id, crate::media::SourceLevel::Thumb).map(|src| lightcraft_pipeline::cull::sharpness(&src)).unwrap_or(0.0);
        if let Some(v) = sig {
            sigs.push((id, v, sharp));
        }
    }
    // union-find
    let mut parent: Vec<usize> = (0..sigs.len()).collect();
    fn find(parent: &mut [usize], mut x: usize) -> usize {
        while parent[x] != x {
            let nxt = parent[x];
            parent[x] = parent[nxt];
            x = nxt;
        }
        x
    }
    for i in 0..sigs.len() {
        for j in (i + 1)..sigs.len() {
            if lightcraft_pipeline::cull::similarity(&sigs[i].1, &sigs[j].1) >= threshold {
                let (a, b) = (find(&mut parent, i), find(&mut parent, j));
                if a != b {
                    parent[b] = a;
                }
            }
        }
    }
    let mut groups: std::collections::HashMap<usize, Vec<usize>> = Default::default();
    for (i, _) in sigs.iter().enumerate() {
        let r = find(&mut parent, i);
        groups.entry(r).or_default().push(i);
    }
    let mut out: Vec<Vec<u64>> = Vec::new();
    let mut ops = Vec::new();
    let mut rejected = 0u32;
    for (_, members) in groups.into_iter().filter(|(_, m)| m.len() > 1) {
        // keeper: sharpest
        let mut order = members;
        order.sort_by(|&a, &b| sigs[b].2.total_cmp(&sigs[a].2));
        out.push(order.iter().map(|&i| sigs[i].0.0).collect());
        if reject {
            for &i in &order[1..] {
                let id = sigs[i].0;
                if s.catalog.photo(id).map(|p| p.flag).unwrap_or_default() != Flag::Reject {
                    ops.push(Op::SetFlag { id, flag: Flag::Reject });
                    rejected += 1;
                }
            }
        }
    }
    out.sort_by_key(|g| std::cmp::Reverse(g.len()));
    if !ops.is_empty() {
        s.commit("Reject Duplicates", Op::Batch { ops })?;
    }
    Ok(json!({"groups": out, "rejected": rejected}))
}
