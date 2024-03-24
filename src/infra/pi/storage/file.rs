pub struct File {
    pub path: String,
    pub header: bool,
    //the name of src column when inputting csv
    //only used when header=true and edgeschema
    pub src_col: Option<String>,
    //the name of dst column when inputting csv
    //only used when header=true and edgeschema
    pub dst_col: Option<String>,
}

impl File {
    pub fn new(
        path: String,
        header: bool,
        src_col: Option<String>,
        dst_col: Option<String>,
    ) -> Self {
        Self {
            path,
            header,
            src_col,
            dst_col,
        }
    }
}

//Note: for neo4j csv, if we use header, each row will be inputted as
// a map. However, if edge_schema.src_vertex_primary_key==edge_schema.dst_vertex_primary_key
//then we cannot use the primary keys to index the map when using header
//Considering the fact that edge_schema.src_vertex_primary_key==edge_schema.dst_vertex_primary_key
// is very common, it is worth adding the option of specifying src_col and dst_col
//to the file struct.
