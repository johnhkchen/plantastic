# T-033-01 Design: DBSCAN Clustering

## Decision: In-crate DBSCAN with kiddo range queries

### Option A: External clustering crate (e.g., `linfa-clustering`)
- **Pro:** Battle-tested implementation, handles edge cases
- **Con:** Heavy dependency (linfa ecosystem pulls ndarray, num-traits, etc.), type conversion overhead (Point → ndarray), limited control over output format
- **Rejected:** Too heavy for a ~50-line algorithm. We already have the spatial index.

### Option B: New `pt-features` crate
- **Pro:** Clean boundary between scan processing and feature extraction
- **Con:** Premature abstraction — we don't know the feature extraction API yet (T-033-02 will define it). Creates a circular dependency problem: pt-features needs `Point` from pt-scan, and pt-scan might want to optionally run clustering in its pipeline.
- **Rejected:** Wait for T-033-02 to define the boundary.

### Option C: `cluster` module in pt-scan (chosen)
- **Pro:** Uses existing `Point` and `BoundingBox` types directly, leverages kiddo already in dependencies, no new crate to wire up, keeps the scan pipeline self-contained
- **Con:** pt-scan grows slightly larger
- **Why chosen:** Clustering obstacle points is a natural extension of the scan pipeline. The module is self-contained (no new dependencies) and the types it produces (`Cluster`) are useful both within pt-scan (enriching `ScanReport`) and downstream (T-033-02 feature candidates).

## Algorithm: DBSCAN with spatial index

Classic DBSCAN (Ester et al., 1996):

```
for each unvisited point P:
    mark P as visited
    neighbors = range_query(P, epsilon)
    if |neighbors| < min_points:
        mark P as noise
    else:
        create new cluster C
        add P to C
        expand_cluster(C, neighbors, epsilon, min_points)

expand_cluster(C, neighbors, epsilon, min_points):
    for each point Q in neighbors:
        if Q not visited:
            mark Q as visited
            Q_neighbors = range_query(Q, epsilon)
            if |Q_neighbors| >= min_points:
                add Q_neighbors to neighbors
        if Q not in any cluster:
            add Q to C
```

Using `kiddo::ImmutableKdTree::within::<SquaredEuclidean>(&pos, eps_squared)` for range queries gives O(n log n) average case.

## Output Types

```rust
pub struct Cluster {
    pub id: u32,
    pub point_indices: Vec<usize>,  // indices into input slice
    pub centroid: [f32; 3],
    pub bbox: BoundingBox,
}

pub struct ClusterConfig {
    pub epsilon: f32,       // neighborhood radius in meters
    pub min_points: usize,  // minimum points to form a cluster
}

pub struct ClusterResult {
    pub clusters: Vec<Cluster>,
    pub noise_indices: Vec<usize>,  // indices of noise points
}
```

### Design Decisions

1. **Indices, not copies.** `point_indices` stores indices into the input `&[Point]` slice. This avoids cloning points and lets downstream code access the original point data (including color).

2. **Noise separated.** Noise points get their own `noise_indices` vec rather than a special cluster. This is cleaner for downstream: T-033-02 can decide whether to inspect noise or discard it.

3. **Centroid computed during clustering.** Average of all member point positions. Cheap to compute incrementally but we'll compute once at the end for simplicity.

4. **BoundingBox reuses existing type.** `BoundingBox::from_points()` already exists but takes `&[Point]`. We'll add a variant or compute inline from positions.

5. **ClusterConfig defaults.** `epsilon: 0.3`, `min_points: 50` per ticket AC. These are tuned for outdoor LiDAR scans where tree trunks are dense clusters and gaps between features are >0.3m.

## Pipeline Integration

Clustering is **not** added to `process_scan()` or `process_scan_timed()` in this ticket. It's a standalone function that callers (like the CLI example or API) invoke on `cloud.obstacles`. Rationale:
- Not all scan consumers need clustering
- ClusterConfig is separate from ScanConfig
- Keeps the existing pipeline untouched — no risk of regression

Future tickets (T-033-02, T-032-03) will wire clustering into the pipeline.

## Testing Strategy

1. **Two well-separated groups → two clusters.** Two dense blobs at (0,0,0) and (5,5,5) with epsilon=0.5, min_points=3.
2. **Sparse noise between groups → noise.** Random scattered points between the two blobs should not form a cluster or merge the blobs.
3. **Single dense group → one cluster.** All points in one blob.
4. **Empty input → empty output.** Zero clusters, zero noise.
5. **Powell & Market validation.** Process the real scan, cluster obstacles, assert exactly 2 clusters (the two tree trunks). This is an integration test that requires the sample PLY file.
