//! Bound preview texture retention. Keep visible cards plus nearby cached cards.
pub fn evictions(loaded:&[bool],visible:&[bool],selected:usize,cap:usize)->Vec<usize>{
 let count=loaded.iter().filter(|&&v|v).count();
 let mut candidates=loaded.iter().enumerate().filter_map(|(i,&v)|(v&&!visible.get(i).copied().unwrap_or(false)&&i!=selected).then_some(i)).collect::<Vec<_>>();
 candidates.sort_by_key(|&i|std::cmp::Reverse(i.abs_diff(selected)));
 candidates.truncate(count.saturating_sub(cap));candidates
}
#[cfg(test)]mod tests{use super::*;#[test]fn retains_visible_and_selected_under_pressure(){let loaded=vec![true;40];let mut visible=vec![false;40];visible[35]=true;let e=evictions(&loaded,&visible,2,24);assert_eq!(e.len(),16);assert!(!e.contains(&2));assert!(!e.contains(&35));assert!(e.contains(&39));}}
