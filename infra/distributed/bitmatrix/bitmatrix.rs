use roaring::RoaringBitmap;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::ops::BitOrAssign;

// ── Phase 1: Constants & Layout ─────────────────────────────────────

pub const TOTAL_BITS: u32 = 32_000;

pub const REGION_0_FINGERPRINT: (u32, u32) = (0, 999);
pub const REGION_1_CONTENT: (u32, u32) = (1_000, 19_999);
pub const REGION_2_CONVERSATION_BP: (u32, u32) = (20_000, 20_999);
pub const REGION_3_CROSS_TURN: (u32, u32) = (21_000, 21_999);
pub const REGION_4_RESERVED: (u32, u32) = (22_000, 25_999);
pub const REGION_5_RESERVED: (u32, u32) = (26_000, 29_999);
pub const REGION_6_PIECE: (u32, u32) = (30_000, 31_999);

pub const ALL_REGIONS: [(& str, (u32, u32)); 7] = [
    ("fingerprint", REGION_0_FINGERPRINT),
    ("content", REGION_1_CONTENT),
    ("conversation_bp", REGION_2_CONVERSATION_BP),
    ("cross_turn", REGION_3_CROSS_TURN),
    ("reserved_a", REGION_4_RESERVED),
    ("reserved_b", REGION_5_RESERVED),
    ("piece", REGION_6_PIECE),
];

// Fingerprint sub-regions
pub const R0_GRAMMAR: (u32, u32) = (0, 127);
pub const R0_INTENT: (u32, u32) = (128, 255);
pub const R0_ENTITY: (u32, u32) = (256, 511);
pub const R0_EMOTIONAL: (u32, u32) = (512, 767);
pub const R0_STRUCTURAL: (u32, u32) = (768, 999);

// Region 6 sub-layout
pub const R6_TOKEN_MAP: (u32, u32) = (30_000, 30_999);
pub const R6_SEQUENCE: (u32, u32) = (31_000, 31_999);

// BP scale offsets within Region 2
pub const BP_SCALE_0: (u32, u32) = (20_000, 20_499);
pub const BP_SCALE_1: (u32, u32) = (20_500, 20_999);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Gas,
    Liquid,
    Crystal,
}

impl Phase {
    pub fn from_density(density: f64) -> Self {
        if density < 0.10 {
            Phase::Gas
        } else if density < 0.50 {
            Phase::Liquid
        } else {
            Phase::Crystal
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Coherence {
    Tight,
    Moderate,
    Diffuse,
}

impl Coherence {
    pub fn from_ratio(layer3_size: u64, layer0_size: u64) -> Self {
        if layer0_size == 0 {
            return Coherence::Diffuse;
        }
        let ratio = layer3_size as f64 / layer0_size as f64;
        if ratio < 2.0 {
            Coherence::Tight
        } else if ratio < 4.0 {
            Coherence::Moderate
        } else {
            Coherence::Diffuse
        }
    }
}

fn region_for_bit(bit: u32) -> Option<(u32, u32)> {
    for &(_, bounds) in &ALL_REGIONS {
        if bit >= bounds.0 && bit <= bounds.1 {
            return Some(bounds);
        }
    }
    None
}

pub fn read_region(bitmap: &RoaringBitmap, region: (u32, u32)) -> RoaringBitmap {
    let mut result = RoaringBitmap::new();
    for bit in bitmap.iter() {
        if bit >= region.0 && bit <= region.1 {
            result.insert(bit);
        } else if bit > region.1 {
            break;
        }
    }
    result
}

pub fn region_cardinality(bitmap: &RoaringBitmap, region: (u32, u32)) -> u64 {
    read_region(bitmap, region).len()
}

pub fn region_is_empty(bitmap: &RoaringBitmap, region: (u32, u32)) -> bool {
    region_cardinality(bitmap, region) == 0
}

// ── Phase 2: Bit Taxonomy ───────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BitRole {
    Sugar,
    Needy,
    Dirty,
    Holy,
    Pocket,
    Compleks,
    Death,
}

// Priority order for classify: Holy > Compleks > Pocket > Sugar > Needy > Dirty > Death
const CLASSIFY_ORDER: [BitRole; 7] = [
    BitRole::Holy,
    BitRole::Compleks,
    BitRole::Pocket,
    BitRole::Sugar,
    BitRole::Needy,
    BitRole::Dirty,
    BitRole::Death,
];

#[derive(Debug, Clone)]
pub struct BitCloud {
    pub sugar: RoaringBitmap,
    pub needy: RoaringBitmap,
    pub dirty: RoaringBitmap,
    pub holy: RoaringBitmap,
    pub pocket: RoaringBitmap,
    pub compleks: RoaringBitmap,
    pub death: RoaringBitmap,
}

impl BitCloud {
    pub fn empty() -> Self {
        Self {
            sugar: RoaringBitmap::new(),
            needy: RoaringBitmap::new(),
            dirty: RoaringBitmap::new(),
            holy: RoaringBitmap::new(),
            pocket: RoaringBitmap::new(),
            compleks: RoaringBitmap::new(),
            death: RoaringBitmap::new(),
        }
    }

    pub fn all_bits(&self) -> RoaringBitmap {
        let mut u = self.sugar.clone();
        u |= &self.needy;
        u |= &self.dirty;
        u |= &self.holy;
        u |= &self.pocket;
        u |= &self.compleks;
        u |= &self.death;
        u
    }

    fn bitmap(&self, role: BitRole) -> &RoaringBitmap {
        match role {
            BitRole::Sugar => &self.sugar,
            BitRole::Needy => &self.needy,
            BitRole::Dirty => &self.dirty,
            BitRole::Holy => &self.holy,
            BitRole::Pocket => &self.pocket,
            BitRole::Compleks => &self.compleks,
            BitRole::Death => &self.death,
        }
    }

    fn bitmap_mut(&mut self, role: BitRole) -> &mut RoaringBitmap {
        match role {
            BitRole::Sugar => &mut self.sugar,
            BitRole::Needy => &mut self.needy,
            BitRole::Dirty => &mut self.dirty,
            BitRole::Holy => &mut self.holy,
            BitRole::Pocket => &mut self.pocket,
            BitRole::Compleks => &mut self.compleks,
            BitRole::Death => &mut self.death,
        }
    }

    pub fn classify(&self, position: u32) -> Option<BitRole> {
        for &role in &CLASSIFY_ORDER {
            if self.bitmap(role).contains(position) {
                return Some(role);
            }
        }
        None
    }

    pub fn promote(&mut self, positions: &[u32], from: BitRole, to: BitRole) {
        for &pos in positions {
            if self.bitmap(from).contains(pos) {
                self.bitmap_mut(from).remove(pos);
                self.bitmap_mut(to).insert(pos);
            }
        }
    }
}

// ── Phase 3: RUBICS Encoding ────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Cube {
    pub region: RoaringBitmap,
    pub uncertainty: RoaringBitmap,
    pub bridge: RoaringBitmap,
    pub identity: RoaringBitmap,
    pub correlation: RoaringBitmap,
    pub sequence: RoaringBitmap,
}

impl Cube {
    pub fn empty() -> Self {
        Self {
            region: RoaringBitmap::new(),
            uncertainty: RoaringBitmap::new(),
            bridge: RoaringBitmap::new(),
            identity: RoaringBitmap::new(),
            correlation: RoaringBitmap::new(),
            sequence: RoaringBitmap::new(),
        }
    }

    pub fn flat(&self) -> RoaringBitmap {
        let mut u = self.region.clone();
        u |= &self.uncertainty;
        u |= &self.bridge;
        u |= &self.identity;
        u |= &self.correlation;
        u |= &self.sequence;
        u
    }
}

#[derive(Debug, Clone)]
pub struct Sugar {
    pub boost: RoaringBitmap,
    pub suppress: RoaringBitmap,
}

impl Sugar {
    pub fn neutral() -> Self {
        Self {
            boost: RoaringBitmap::new(),
            suppress: RoaringBitmap::new(),
        }
    }

    pub fn mirror(&self) -> Spice {
        Spice {
            amplify: self.boost.clone(),
            filter_out: self.suppress.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Spice {
    pub amplify: RoaringBitmap,
    pub filter_out: RoaringBitmap,
}

impl Spice {
    pub fn neutral() -> Self {
        Self {
            amplify: RoaringBitmap::new(),
            filter_out: RoaringBitmap::new(),
        }
    }

    pub fn mirror(&self) -> Sugar {
        Sugar {
            boost: self.amplify.clone(),
            suppress: self.filter_out.clone(),
        }
    }
}

pub fn sweeten(bits: &RoaringBitmap, sugar: &Sugar) -> RoaringBitmap {
    let mut r = bits | &sugar.boost;
    r -= &sugar.suppress;
    r
}

pub fn flavor(bits: &RoaringBitmap, spice: &Spice) -> RoaringBitmap {
    let mut r = bits | &spice.amplify;
    r -= &spice.filter_out;
    r
}

pub fn rubics_encode(bits: &RoaringBitmap, sugar: &Sugar) -> Cube {
    Cube {
        region: sweeten(bits, sugar),
        uncertainty: RoaringBitmap::new(),
        bridge: RoaringBitmap::new(),
        identity: RoaringBitmap::new(),
        correlation: RoaringBitmap::new(),
        sequence: RoaringBitmap::new(),
    }
}

pub fn rubics_decode(cube: &Cube, spice: &Spice) -> RoaringBitmap {
    flavor(&cube.flat(), spice)
}

/// Encode+decode in both directions; intersection = bits that survive both passes.
pub fn dance(bits: &RoaringBitmap, sugar: &Sugar, spice: &Spice) -> RoaringBitmap {
    let forward = rubics_decode(&rubics_encode(bits, sugar), spice);
    let mirror_sugar = spice.mirror();
    let mirror_spice = sugar.mirror();
    let backward = rubics_decode(&rubics_encode(bits, &mirror_sugar), &mirror_spice);
    forward & backward
}

// ── Phase 4: Field Layers ───────────────────────────────────────────

pub const NUM_LAYERS: usize = 4;

#[derive(Debug, Clone)]
pub struct FieldLayers {
    pub exact: RoaringBitmap,
    pub bridges: RoaringBitmap,
    pub peripheral: RoaringBitmap,
    layers_cache: Option<[RoaringBitmap; NUM_LAYERS]>,
}

impl FieldLayers {
    pub fn new(exact: RoaringBitmap, bridges: RoaringBitmap, peripheral: RoaringBitmap) -> Self {
        Self { exact, bridges, peripheral, layers_cache: None }
    }

    pub fn empty() -> Self {
        Self::new(RoaringBitmap::new(), RoaringBitmap::new(), RoaringBitmap::new())
    }

    pub fn layer(&self, idx: usize) -> Option<&RoaringBitmap> {
        match idx {
            0 => Some(&self.exact),
            2 => Some(&self.bridges),
            3 => Some(&self.peripheral),
            _ => None, // L1 computed at inference time
        }
    }

    pub fn popcount(&self) -> u64 {
        self.exact.len()
    }

    pub fn total_field_size(&self) -> u64 {
        self.exact.len() + self.bridges.len() + self.peripheral.len()
    }

    pub fn density_at(&self, position: u32) -> u8 {
        let mut d = 0u8;
        if self.exact.contains(position) { d += 1; }
        if self.bridges.contains(position) { d += 1; }
        if self.peripheral.contains(position) { d += 1; }
        d
    }

    pub fn coherence(&self) -> Coherence {
        Coherence::from_ratio(self.peripheral.len(), self.exact.len())
    }
}

pub fn adaptive_dilation_radius(store_size: usize) -> u32 {
    if store_size < 100 {
        3
    } else {
        let shift = (store_size as f64 / 100.0).log2() as u32;
        3u32.saturating_sub(shift).max(1)
    }
}

pub fn dilate_bounded(bitmap: &RoaringBitmap, radius: u32) -> RoaringBitmap {
    if radius == 0 {
        return bitmap.clone();
    }
    let mut result = RoaringBitmap::new();
    for bit in bitmap.iter() {
        if let Some((r_start, r_end)) = region_for_bit(bit) {
            let lo = r_start.max(bit.saturating_sub(radius));
            let hi = r_end.min(bit + radius);
            result.insert_range(lo..=hi);
        }
    }
    result
}

pub fn dilate(bitmap: &RoaringBitmap, radius: u32) -> RoaringBitmap {
    if radius == 0 {
        return bitmap.clone();
    }
    let mut result = RoaringBitmap::new();
    for bit in bitmap.iter() {
        let lo = bit.saturating_sub(radius);
        let hi = bit + radius;
        result.insert_range(lo..=hi);
    }
    result
}

// ── Phase 5: Memory ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Memory {
    pub memory_id: u32,
    pub field: FieldLayers,
    pub raw_text: String,
    pub timestamp: f64,
    pub turn_count: u32,
    pub mera_fine: Option<MeraFine>,
    pub mera_coarse: Option<MeraCoarse>,
    pub braid: Option<Braid>,
}

impl Memory {
    pub fn popcount(&self) -> u64 {
        self.field.popcount()
    }

    pub fn region_bits(&self, region: (u32, u32)) -> RoaringBitmap {
        read_region(&self.field.exact, region)
    }
}

#[derive(Debug, Clone)]
pub struct MemoryStore {
    memories: BTreeMap<u32, Memory>,
    next_id: u32,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self { memories: BTreeMap::new(), next_id: 0 }
    }

    pub fn insert(&mut self, mut mem: Memory) -> u32 {
        let id = self.next_id;
        mem.memory_id = id;
        self.memories.insert(id, mem);
        self.next_id += 1;
        id
    }

    pub fn get(&self, id: u32) -> Option<&Memory> {
        self.memories.get(&id)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut Memory> {
        self.memories.get_mut(&id)
    }

    pub fn all_ids(&self) -> Vec<u32> {
        self.memories.keys().copied().collect()
    }

    pub fn len(&self) -> usize {
        self.memories.len()
    }

    pub fn is_empty(&self) -> bool {
        self.memories.is_empty()
    }
}

// ── Phase 6: BitMatrix Batch Inference ──────────────────────────────

#[derive(Debug, Clone)]
pub struct BitMatrix {
    rows: Vec<RoaringBitmap>,
    id_map: Vec<u32>,
    cols: u32,
}

impl BitMatrix {
    pub fn build(store: &MemoryStore) -> Self {
        let ids = store.all_ids();
        let mut rows = Vec::with_capacity(ids.len());
        let mut max_col: u32 = 0;

        for &id in &ids {
            if let Some(mem) = store.get(id) {
                if let Some(m) = mem.field.exact.max() {
                    max_col = max_col.max(m);
                }
                rows.push(mem.field.exact.clone());
            } else {
                rows.push(RoaringBitmap::new());
            }
        }

        Self {
            rows,
            id_map: ids,
            cols: if max_col > 0 { max_col + 1 } else { TOTAL_BITS },
        }
    }

    pub fn binary_matmul(&self, query: &RoaringBitmap) -> Vec<u32> {
        self.rows
            .iter()
            .map(|row| (row & query).len() as u32)
            .collect()
    }

    pub fn query_overlaps(&self, stimulus: &RoaringBitmap) -> Vec<(u32, u32)> {
        let counts = self.binary_matmul(stimulus);
        self.id_map.iter().copied().zip(counts).collect()
    }

    pub fn query_multi_head(&self, heads: &[RoaringBitmap]) -> Vec<Vec<(u32, u32)>> {
        heads.iter().map(|h| self.query_overlaps(h)).collect()
    }
}

// ── Phase 7: WAIStore Foundation ────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FrozenNumber {
    pub exists: RoaringBitmap,
    pub planes: Vec<RoaringBitmap>,
}

impl FrozenNumber {
    pub fn empty() -> Self {
        Self {
            exists: RoaringBitmap::new(),
            planes: Vec::new(),
        }
    }

    pub fn get(&self, row_id: u32) -> Option<u64> {
        if !self.exists.contains(row_id) {
            return None;
        }
        let mut val = 0u64;
        for (i, plane) in self.planes.iter().enumerate() {
            if plane.contains(row_id) {
                val |= 1u64 << i;
            }
        }
        Some(val)
    }
}

#[derive(Debug, Clone)]
struct MutableNumber {
    exists: RoaringBitmap,
    planes: Vec<RoaringBitmap>,
}

impl MutableNumber {
    fn new() -> Self {
        Self {
            exists: RoaringBitmap::new(),
            planes: Vec::new(),
        }
    }

    fn add(&mut self, row_id: u32, value: u64) {
        self.exists.insert(row_id);
        let bits_needed = if value == 0 { 1 } else { 64 - value.leading_zeros() as usize };
        while self.planes.len() < bits_needed {
            self.planes.push(RoaringBitmap::new());
        }
        for (i, plane) in self.planes.iter_mut().enumerate() {
            if value & (1u64 << i) != 0 {
                plane.insert(row_id);
            }
        }
    }

    fn freeze(self) -> FrozenNumber {
        FrozenNumber {
            exists: self.exists,
            planes: self.planes,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub rows: RoaringBitmap,
    pub deleted: RoaringBitmap,
    pub predicates: HashMap<String, RoaringBitmap>,
    pub numbers: HashMap<String, FrozenNumber>,
}

impl Segment {
    fn empty() -> Self {
        Self {
            rows: RoaringBitmap::new(),
            deleted: RoaringBitmap::new(),
            predicates: HashMap::new(),
            numbers: HashMap::new(),
        }
    }

    fn visible_rows(&self) -> RoaringBitmap {
        &self.rows - &self.deleted
    }
}

#[derive(Debug, Clone)]
pub struct WAIStore {
    head_rows: RoaringBitmap,
    head_deleted: RoaringBitmap,
    head_predicates: HashMap<String, RoaringBitmap>,
    head_numbers: HashMap<String, MutableNumber>,
    segments: Vec<Segment>,
    next_id: u32,
}

impl WAIStore {
    pub fn new() -> Self {
        Self {
            head_rows: RoaringBitmap::new(),
            head_deleted: RoaringBitmap::new(),
            head_predicates: HashMap::new(),
            head_numbers: HashMap::new(),
            segments: Vec::new(),
            next_id: 0,
        }
    }

    pub fn append(
        &mut self,
        tags: &[&str],
        numbers: &[(&str, u64)],
        row_id: Option<u32>,
    ) -> u32 {
        let id = row_id.unwrap_or_else(|| {
            let id = self.next_id;
            self.next_id += 1;
            id
        });
        if row_id.is_some() {
            self.next_id = self.next_id.max(id + 1);
        }

        self.head_rows.insert(id);
        for &tag in tags {
            self.head_predicates
                .entry(tag.to_string())
                .or_insert_with(RoaringBitmap::new)
                .insert(id);
        }
        for &(name, val) in numbers {
            self.head_numbers
                .entry(name.to_string())
                .or_insert_with(MutableNumber::new)
                .add(id, val);
        }
        id
    }

    pub fn delete(&mut self, row_id: u32) {
        self.head_deleted.insert(row_id);
    }

    pub fn seal(&mut self) -> usize {
        let mut predicates = HashMap::new();
        for (k, v) in self.head_predicates.drain() {
            predicates.insert(k, v);
        }
        let mut numbers = HashMap::new();
        for (k, v) in self.head_numbers.drain() {
            numbers.insert(k, v.freeze());
        }

        let seg = Segment {
            rows: std::mem::replace(&mut self.head_rows, RoaringBitmap::new()),
            deleted: std::mem::replace(&mut self.head_deleted, RoaringBitmap::new()),
            predicates,
            numbers,
        };
        self.segments.push(seg);
        self.segments.len()
    }

    pub fn rows(&self) -> RoaringBitmap {
        let mut visible = &self.head_rows - &self.head_deleted;
        for seg in &self.segments {
            visible |= &seg.visible_rows();
        }
        // Apply head-level deletes to sealed segments too
        visible -= &self.head_deleted;
        visible
    }

    pub fn predicate(&self, name: &str) -> RoaringBitmap {
        let mut result = self
            .head_predicates
            .get(name)
            .cloned()
            .unwrap_or_default();
        for seg in &self.segments {
            if let Some(p) = seg.predicates.get(name) {
                result |= p;
            }
        }
        result &= &self.rows();
        result
    }

    pub fn query_and(&self, a: &RoaringBitmap, b: &RoaringBitmap) -> RoaringBitmap {
        a & b
    }

    pub fn query_or(&self, a: &RoaringBitmap, b: &RoaringBitmap) -> RoaringBitmap {
        a | b
    }

    pub fn query_xor(&self, a: &RoaringBitmap, b: &RoaringBitmap) -> RoaringBitmap {
        a ^ b
    }

    pub fn query_diff(&self, a: &RoaringBitmap, b: &RoaringBitmap) -> RoaringBitmap {
        a - b
    }

    pub fn query_complement(&self, a: &RoaringBitmap) -> RoaringBitmap {
        &self.rows() - a
    }

    pub fn compact(&mut self) {
        if self.segments.is_empty() {
            return;
        }
        self.seal();

        let mut rows = RoaringBitmap::new();
        let mut predicates: HashMap<String, RoaringBitmap> = HashMap::new();
        let mut numbers: HashMap<String, MutableNumber> = HashMap::new();
        let mut all_deleted = RoaringBitmap::new();

        for seg in self.segments.drain(..) {
            rows |= &seg.rows;
            all_deleted |= &seg.deleted;
            for (k, v) in seg.predicates {
                predicates.entry(k).or_insert_with(RoaringBitmap::new).bitor_assign(&v);
            }
            for (k, frozen) in seg.numbers {
                let mn = numbers.entry(k).or_insert_with(MutableNumber::new);
                for row_id in frozen.exists.iter() {
                    if let Some(val) = frozen.get(row_id) {
                        mn.add(row_id, val);
                    }
                }
            }
        }

        // Remove tombstoned rows
        rows -= &all_deleted;
        for v in predicates.values_mut() {
            *v -= &all_deleted;
        }

        let mut frozen_numbers = HashMap::new();
        for (k, mn) in numbers {
            let mut f = mn.freeze();
            f.exists -= &all_deleted;
            frozen_numbers.insert(k, f);
        }

        self.segments.push(Segment {
            rows,
            deleted: RoaringBitmap::new(),
            predicates,
            numbers: frozen_numbers,
        });
    }

    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }
}

// ── Phase 8: MERA + Braid ───────────────────────────────────────────

pub const MERA_MAX_LEVELS: usize = 14;
pub const MERA_FINE_COARSE_BOUNDARY: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BinaryTensor {
    pub bits: u16,
}

impl BinaryTensor {
    pub fn new(bits: u16) -> Self {
        Self { bits }
    }

    pub fn from_pair(bit_a: bool, bit_b: bool, same_turn: bool) -> Self {
        let mut bits: u16 = 0;
        if bit_a { bits |= 0b0001; }
        if bit_b { bits |= 0b0010; }
        if bit_a && bit_b { bits |= 0b0100; }
        if same_turn { bits |= 0b1000; }
        Self { bits }
    }

    /// XNOR + popcount: agreement score 0-16
    pub fn contract(&self, other: &BinaryTensor) -> u32 {
        let xnor = !(self.bits ^ other.bits);
        xnor.count_ones()
    }
}

#[derive(Debug, Clone)]
pub struct MeraFine {
    levels: Vec<Vec<BinaryTensor>>,
}

impl MeraFine {
    pub fn new(levels: Vec<Vec<BinaryTensor>>) -> Self {
        Self { levels }
    }

    pub fn num_levels(&self) -> usize {
        self.levels.len()
    }

    pub fn level(&self, idx: usize) -> &[BinaryTensor] {
        self.levels.get(idx).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

#[derive(Debug, Clone)]
pub struct MeraCoarse {
    levels: Vec<Vec<BinaryTensor>>,
}

impl MeraCoarse {
    pub fn new(levels: Vec<Vec<BinaryTensor>>) -> Self {
        Self { levels }
    }

    pub fn num_levels(&self) -> usize {
        self.levels.len()
    }

    pub fn level(&self, idx: usize) -> &[BinaryTensor] {
        self.levels.get(idx).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

pub fn build_mera_from_bits(positions: &[u32], same_turn: bool) -> (MeraFine, MeraCoarse) {
    let mut levels: Vec<Vec<BinaryTensor>> = Vec::new();

    // Level 0: pair adjacent bit positions into tensors
    let mut current: Vec<BinaryTensor> = positions
        .chunks(2)
        .map(|chunk| {
            let a = chunk[0] != 0;
            let b = if chunk.len() > 1 { chunk[1] != 0 } else { false };
            BinaryTensor::from_pair(a, b, same_turn)
        })
        .collect();

    if current.is_empty() {
        current.push(BinaryTensor::new(0));
    }
    levels.push(current.clone());

    // Contract pairwise up to MAX_LEVELS
    for _ in 1..MERA_MAX_LEVELS {
        if current.len() <= 1 {
            break;
        }
        let next: Vec<BinaryTensor> = current
            .chunks(2)
            .map(|chunk| {
                if chunk.len() == 2 {
                    let agreement = chunk[0].contract(&chunk[1]);
                    BinaryTensor::new(agreement as u16)
                } else {
                    chunk[0]
                }
            })
            .collect();
        levels.push(next.clone());
        current = next;
    }

    // Split at boundary
    let boundary = MERA_FINE_COARSE_BOUNDARY.min(levels.len());
    let fine_levels = levels[..boundary].to_vec();
    let coarse_levels = if boundary < levels.len() {
        levels[boundary..].to_vec()
    } else {
        Vec::new()
    };

    (MeraFine::new(fine_levels), MeraCoarse::new(coarse_levels))
}

pub fn contract_levels(a: &MeraFine, b: &MeraFine) -> HashMap<usize, u32> {
    let mut result = HashMap::new();
    let n = a.num_levels().min(b.num_levels());
    for lvl in 0..n {
        let la = a.level(lvl);
        let lb = b.level(lvl);
        let pairs = la.len().min(lb.len());
        if pairs == 0 {
            continue;
        }
        let total: u32 = la.iter().zip(lb.iter()).map(|(ta, tb)| ta.contract(tb)).sum();
        result.insert(lvl, total / pairs as u32);
    }
    result
}

const MAX_CLUSTERS: usize = 64;
const MAX_CROSSINGS: usize = 2048;

#[derive(Debug, Clone)]
pub struct Braid {
    pub active_clusters: Vec<String>,
    pub crossings: BTreeSet<(String, String)>,
    pub signature: u32,
}

impl Braid {
    pub fn empty() -> Self {
        Self {
            active_clusters: Vec::new(),
            crossings: BTreeSet::new(),
            signature: 0,
        }
    }

    /// Jaccard similarity on crossings
    pub fn compare(&self, other: &Braid) -> f64 {
        if self.crossings.is_empty() && other.crossings.is_empty() {
            return 1.0;
        }
        let intersection = self.crossings.intersection(&other.crossings).count();
        let union = self.crossings.union(&other.crossings).count();
        if union == 0 { 1.0 } else { intersection as f64 / union as f64 }
    }

    /// XNOR + popcount on u32 signature: 0-32 agreement
    pub fn fast_compare(&self, other: &Braid) -> u32 {
        let xnor = !(self.signature ^ other.signature);
        xnor.count_ones()
    }

    pub fn strand_count(&self) -> usize {
        self.active_clusters.len()
    }

    pub fn crossing_count(&self) -> usize {
        self.crossings.len()
    }

    pub fn has_cluster(&self, name: &str) -> bool {
        self.active_clusters.iter().any(|c| c == name)
    }

    pub fn shared_clusters(&self, other: &Braid) -> Vec<String> {
        self.active_clusters
            .iter()
            .filter(|c| other.has_cluster(c))
            .cloned()
            .collect()
    }
}

// ── Phase 9: Pipeline Enums ─────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Rest,
    Shadow,
    Signal,
    Care,
    Next,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vote {
    Left,
    Center,
    Right,
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_boundaries_cover_full_space() {
        let mut covered = RoaringBitmap::new();
        for &(_, (start, end)) in &ALL_REGIONS {
            covered.insert_range(start..=end);
        }
        assert_eq!(covered.len(), TOTAL_BITS as u64);
    }

    #[test]
    fn bitcloud_promote() {
        let mut cloud = BitCloud::empty();
        cloud.sugar.insert(10);
        cloud.sugar.insert(20);
        cloud.promote(&[10], BitRole::Sugar, BitRole::Holy);

        assert!(!cloud.sugar.contains(10));
        assert!(cloud.holy.contains(10));
        assert!(cloud.sugar.contains(20));
        assert_eq!(cloud.classify(10), Some(BitRole::Holy));
    }

    #[test]
    fn rubics_dance_holy_bits() {
        let mut bits = RoaringBitmap::new();
        bits.insert_range(100..=110);

        let sugar = Sugar::neutral();
        let spice = Spice::neutral();
        let holy = dance(&bits, &sugar, &spice);
        // With neutral modifiers, all bits survive both directions
        assert_eq!(holy, bits);
    }

    #[test]
    fn dilate_bounded_stays_in_region() {
        let mut bm = RoaringBitmap::new();
        // Bit at start of content region
        bm.insert(REGION_1_CONTENT.0);
        let dilated = dilate_bounded(&bm, 5);
        // Should not leak into fingerprint region
        assert!(!dilated.contains(REGION_0_FINGERPRINT.1));
        assert!(dilated.contains(REGION_1_CONTENT.0));
        assert!(dilated.contains(REGION_1_CONTENT.0 + 5));
    }

    #[test]
    fn field_density() {
        let mut exact = RoaringBitmap::new();
        exact.insert(5000);
        let mut bridges = RoaringBitmap::new();
        bridges.insert(5000);
        let peripheral = RoaringBitmap::new();

        let field = FieldLayers::new(exact, bridges, peripheral);
        assert_eq!(field.density_at(5000), 2);
        assert_eq!(field.density_at(5001), 0);
    }

    #[test]
    fn bitmatrix_query_overlaps() {
        let mut store = MemoryStore::new();

        let mut f1 = FieldLayers::empty();
        f1.exact.insert_range(1000..=1010);
        store.insert(Memory {
            memory_id: 0,
            field: f1,
            raw_text: String::new(),
            timestamp: 0.0,
            turn_count: 1,
            mera_fine: None,
            mera_coarse: None,
            braid: None,
        });

        let mut f2 = FieldLayers::empty();
        f2.exact.insert_range(1005..=1015);
        store.insert(Memory {
            memory_id: 0,
            field: f2,
            raw_text: String::new(),
            timestamp: 1.0,
            turn_count: 1,
            mera_fine: None,
            mera_coarse: None,
            braid: None,
        });

        let matrix = BitMatrix::build(&store);
        let mut query = RoaringBitmap::new();
        query.insert_range(1005..=1010);

        let overlaps = matrix.query_overlaps(&query);
        // Memory 0: bits 1005-1010 overlap (6 bits)
        assert_eq!(overlaps[0].1, 6);
        // Memory 1: bits 1005-1010 overlap (6 bits)
        assert_eq!(overlaps[1].1, 6);
    }

    #[test]
    fn waistore_roundtrip() {
        let mut store = WAIStore::new();
        store.append(&["person", "active"], &[("age", 25)], None);
        store.append(&["person"], &[("age", 30)], None);
        store.append(&["bot"], &[("age", 0)], None);

        let persons = store.predicate("person");
        assert_eq!(persons.len(), 2);

        let active = store.predicate("active");
        assert_eq!(active.len(), 1);

        store.delete(1);
        let persons_after = store.predicate("person");
        assert_eq!(persons_after.len(), 1);

        store.seal();
        assert_eq!(store.segment_count(), 1);
        assert_eq!(store.rows().len(), 2);
    }

    #[test]
    fn mera_contract() {
        let a = BinaryTensor::new(0b1010_1010_1010_1010);
        let b = BinaryTensor::new(0b1010_1010_1010_1010);
        assert_eq!(a.contract(&b), 16); // perfect agreement

        let c = BinaryTensor::new(0b0101_0101_0101_0101);
        assert_eq!(a.contract(&c), 0); // total disagreement
    }

    #[test]
    fn braid_compare() {
        let mut a = Braid::empty();
        a.crossings.insert(("body".into(), "emotion".into()));
        a.crossings.insert(("food".into(), "body".into()));
        a.signature = 0xDEAD_BEEF;

        let mut b = Braid::empty();
        b.crossings.insert(("body".into(), "emotion".into()));
        b.signature = 0xDEAD_BEEE;

        assert!(a.compare(&b) > 0.0 && a.compare(&b) < 1.0);
        assert!(a.fast_compare(&b) > 28); // signatures differ by 1 bit
    }

    #[test]
    fn adaptive_radius_shrinks() {
        assert_eq!(adaptive_dilation_radius(10), 3);
        assert_eq!(adaptive_dilation_radius(100), 3);
        assert!(adaptive_dilation_radius(1000) < 3);
        assert!(adaptive_dilation_radius(10000) >= 1);
    }
}
