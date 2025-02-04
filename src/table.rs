use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Foreign{
    pub schema:String,
    pub table:String,
    pub column:String,
}

#[derive(Debug, Clone)]
pub struct Column{
    pub name:String,
    /// the generic data type, ie: u32, f64, string
    pub data_type:String,
    /// the database data type of this column, ie: int, numeric, character varying
    pub db_data_type:String,
    pub is_primary:bool,
    pub is_unique:bool,
    pub default:Option<String>,
    pub comment:Option<String>,
    pub not_null:bool,
    pub foreign:Option<Foreign>,
    ///determines if the column is inherited from the parent table
    pub is_inherited:bool,
}

impl Column{

    fn is_keyword(str:&str)->bool{
        let keyword = ["type", "yield", "macro"];
        keyword.contains(&str)
    }


    ///some column names may be a rust reserve keyword, so have to correct them
    pub fn corrected_name(&self)->String{
        if Self::is_keyword(&self.name){
            println!("Warning: {} is rust reserved keyword", self.name);
            return format!("{}_",self.name);
        }
        self.name.to_string()
    }
    
     pub fn displayname(&self)->String{
         let clean_name = self.clean_name();
         clean_name.replace("_", " ")
    }
    
    /// presentable display names, such as removing the ids if it ends with one
    fn clean_name(&self)->String{
        if self.name.ends_with("_id"){
            return self.name.trim_right_matches("_id").to_string();
        }
        self.name.to_string()
    }
    
    /// shorten, compress the name based on the table it points to
    /// parent_organization_id becomes parent
    pub fn condense_name(&self)->String{
        let clean_name = self.clean_name();
        if self.foreign.is_some(){
            let foreign = &self.foreign.clone().unwrap();
            if clean_name.len() > foreign.table.len(){
                return clean_name
                        .trim_right_matches(&foreign.table)
                        .trim_right_matches("_")
                        .to_string();
            }
        }
        clean_name
    }

}


impl fmt::Display for Column {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq for Column{
    fn eq(&self, other: &Self) -> bool{
        self.name == other.name
     }

    fn ne(&self, other: &Self) -> bool {
        self.name != other.name
    }
}

/// trait for table definition
pub trait IsTable{
    fn table()->Table;
}

/// all referenced table used in context
pub struct RefTable<'a>{
    /// the table being referred
    pub table: &'a Table,
    /// the referring column, applicable to direct has_one
    column: Option<&'a Column>,
    linker_table: Option<&'a Table>,
    pub is_ext: bool,
    pub is_has_one: bool,
    pub is_has_many: bool,
    pub is_direct: bool,
}

/// FIXME need more terse and ergonomic handling of conflicting member names
impl <'a>RefTable<'a>{
    
    /// return the appropriate member name of this reference
    /// when used with the table in context
    /// will have to use another name if the comed up name
    /// already in the column names
    /// 1. the concise name of the referred/referrring table
    /// 2. the name of the referred/referring table
    /// 3. the appended column_name and the table name
    /// 4. the table_name appended with HasMany, or HasOne
    /// 1:1, 1:M, M:M
    /// 11, 1m mm
    pub fn member_name(&self, used_in_table:&Table)->String{
        let has_conflict = false;
        if self.is_has_one{
            if has_conflict{
                let suffix = "_1";
                return format!("{}{}",self.column.unwrap().name, suffix);
            }else{
                return self.column.unwrap().condense_name();
            }
        }
        if self.is_ext{
            if has_conflict{
                let suffix = "_1";
                return format!("{}{}",self.table.name, suffix);
            }else{
                return self.table.condensed_member_name(used_in_table);
            }
        }
        if self.is_has_many && self.is_direct{
            if has_conflict{
                let suffix = "_1m";
                return format!("{}{}",self.table.name, suffix);
            }else{
                return self.table.name.to_string();
            }
            
        }
         if self.is_has_many && !self.is_direct{
             if has_conflict{
                let suffix = "_mm";
                return format!("{}{}",self.table.name, suffix);
            }else{
                return self.table.name.to_string();
            }
        }
        unreachable!();
    }
}


#[derive(Debug)]
#[derive(Clone)]
pub struct Table{

    ///which schema this belongs
    pub schema:String,

    ///the table name
    pub name:String,

    ///the parent table of this table when inheriting (>= postgresql 9.3)
    /// [FIXME] need to tell which schema this parent table belongs
    /// there might be same table in different schemas
    pub parent_table:Option<String>,

    ///what are the other table that inherits this
    /// [FIXME] need to tell which schema this parent table belongs
    /// there might be same table in different schemas
    pub sub_table:Vec<String>,

    ///comment of this table
    pub comment:Option<String>,

    ///columns of this table
    pub columns:Vec<Column>,

}
impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq for Table{
    fn eq(&self, other: &Self) -> bool{
        self.name == other.name && self.schema == other.schema
     }

    fn ne(&self, other: &Self) -> bool {
        self.name != other.name || self.schema != other.schema
    }
}


impl Table{
    
    /// return the long name of the table using schema.table_name
    pub fn complete_name(&self)->String{
        format!("{}.{}", self.schema, self.name)
    }
    
    /// capitalize the first later, if there is underscore remove it then capitalize the next letter
    pub fn struct_name(&self)->String{
        let mut struct_name = String::new();
        for i in self.name.split('_'){
                struct_name.push_str(&capitalize(i));
        }
        struct_name
    }
    
    /// get the display name of this table
    /// product_availability -> Product Availability
    pub fn displayname(&self)->String{
        let mut display_name = String::new();
        for i in self.name.split('_'){
            display_name.push_str(&capitalize(i));
            display_name.push_str(" ");
        }
        display_name.trim().to_string()
    }
    
    /// get a shorter display name of a certain table
    /// when being refered to this table
    /// example product.product_availability -> Availability
    /// user.user_info -> Info
    pub fn condensed_displayname(&self, table:&Table)->String{
        if self.name.len() > table.name.len(){
            let mut concise_name = String::new();
            for i in self.name.split('_'){
                if table.name != i{
                    concise_name.push_str(&capitalize(i));
                    concise_name.push_str(" ");
                }
            }
            return concise_name.trim().to_string()    
        }else{
            return self.displayname();
        }
    }
    
   /// remove plural names such as users to user
   fn clean_name(&self)->String{
        if self.name.ends_with("s"){
            return self.name.trim_right_matches("s").to_string();
        }
        if self.name.ends_with("ies"){
            return self.name.trim_right_matches("y").to_string();
        }
        self.name.to_string()
    }
    
    /// get a condensed name of this table when used in contex with another table
    pub fn condensed_member_name(&self, used_in_table:&Table)->String{
        if self.name.len() > used_in_table.name.len(){
            let mut concise_name = String::new();
            let used_in_tablename = used_in_table.clean_name();
            for i in self.name.split('_'){
                if used_in_tablename != i{
                    concise_name.push_str(i);
                    concise_name.push_str("_");
                }
            }
            return concise_name.trim_right_matches("_").to_string()    
        }else{
            return self.name.to_string();
        }
    }
    
    /// determine if this table has a colum named
    pub fn has_column_name(&self, column:&str)->bool{
        for c in &self.columns{
            if c.name == column.clone(){
                return true;
            }
        }
        false
    }

    /// return all the primary columns of this table
    pub fn primary_columns(&self)->Vec<&Column>{
        let mut primary_columns = Vec::new();
        for c in &self.columns{
            if c.is_primary{
                primary_columns.push(c);
            }
        }
        primary_columns.sort_by(|a, b| a.name.cmp(&b.name));
        primary_columns
    }
    
    /// return all the columns of this table excluding the inherited columns
    pub fn uninherited_columns(&self)->Vec<&Column>{
        let mut included = Vec::new();
        let mut uninherited_columns = Vec::new();
        for c in &self.columns{
            if !c.is_inherited && !included.contains(&&c.name){
                uninherited_columns.push(c);
                included.push(&c.name);
            }
        }
        uninherited_columns.sort_by(|a, b| a.name.cmp(&b.name));
        uninherited_columns
    }

    /// return all the inherited columns
    pub fn inherited_columns(&self)->Vec<&Column>{
        let mut included = Vec::new();
        let mut inherited_columns = Vec::new();
        for c in &self.columns{
            if c.is_inherited && !included.contains(&&c.name){
                inherited_columns.push(c);
                included.push(&c.name);
            }
        }
        inherited_columns.sort_by(|a, b| a.name.cmp(&b.name));
        inherited_columns
    }

    /// check to see if the column is a primary or not
    /// the Column.is_primary property is not reliable since it also list down the foreign key
    /// which makes it 2 entries in the table
    pub fn is_primary(&self, column_name:&str)->bool{
        for p in self.primary_columns(){
            if p.name == column_name {
                return true;
            }
        }
        false
    }

    /// return all the unique keys of this table
    pub fn unique_columns(&self)->Vec<&Column>{
        let mut unique_columns = Vec::new();
        for c in &self.columns{
            if c.is_unique{
                unique_columns.push(c);
            }
        }
        unique_columns.sort_by(|a, b| a.name.cmp(&b.name));
        unique_columns
    }

    pub fn foreign_columns(&self)->Vec<&Column>{
        let mut columns = Vec::new();
        for c in &self.columns{
            if c.foreign.is_some(){
                columns.push(c);
            }
        }
        columns.sort_by(|a, b| a.name.cmp(&b.name));
        columns
    }

    /// return the first match of table name regardless of which schema it belongs to.
    /// get the table definition using the table name from an array of table object
    /// [FIXME] Needs to have a more elegant solution by using HashMap
    pub fn get_table<'a>(schema: &str, table_name:&str, tables: &'a Vec<Table>)->&'a Table{
        for t in tables{
            if t.schema == schema && t.name == table_name{
                return t;
            }
        }
        panic!("Table {} is not on the list can not be found", table_name);
    }



    /// get all the tables that is referred by this table
    /// get has_one
    fn referred_tables<'a>(&'a self, tables:&'a Vec<Table>)->Vec<(&'a Column, &'a Table)>{
        let mut referred_tables = Vec::new();
        for c in &self.columns{
            if c.foreign.is_some(){
                let ft = &c.foreign.clone().unwrap();
                let ftable = Self::get_table(&ft.schema, &ft.table, tables);
                referred_tables.push((c, ftable));
            }
        }
        referred_tables
    }

    /// has_many_direct
    /// get all other tables that is refering to this table
    /// when any column of a table refers to this table
    /// get_has_many
    fn referring_tables<'a>(&self, tables: &'a Vec<Table>)->Vec<(&'a Table, &'a Column)>{
        let mut referring = Vec::new();
        for t in tables{
            for c in &t.columns{
                if c.foreign.is_some(){
                    if &self.name == &c.foreign.clone().unwrap().table{
                        referring.push((t, c));
                    }
                }
            }
        }
        referring
    }
    
    pub fn get_all_referenced_table<'a>(&'a self, all_tables:&'a Vec<Table>)->Vec<RefTable>{
        let mut referenced_tables = vec![];
        
        let has_one = self.referred_tables(all_tables);
        for (column, table) in has_one{
            let ref_table = RefTable{
                                table: table,
                                column: Some(column),
                                linker_table: None,
                                is_has_one: true,
                                is_ext: false,
                                is_has_many: false,
                                is_direct: true,
                            };
            referenced_tables.push(ref_table);
        }
        
        
        let extension_tables = self.extension_tables(all_tables);
        for ext in &extension_tables{
             let ref_table = RefTable{
                                table: ext,
                                column: None,
                                linker_table: None,
                                is_has_one: false,
                                is_ext: true,
                                is_has_many: false,
                                is_direct: true,
                            };
            referenced_tables.push(ref_table);
        }
        
        let has_many_direct = self.referring_tables(all_tables);
        let mut included_has_many = vec![];
        for (hd,column) in has_many_direct{
            if !hd.is_linker_table() &&
                !extension_tables.contains(&hd) &&
                !included_has_many.contains(&hd){
                let ref_table = RefTable{
                            table: hd,
                            column: Some(column),
                            linker_table: None,
                            is_has_one: false,
                            is_ext: false,
                            is_has_many: true,
                            is_direct: true,
                        };
                referenced_tables.push(ref_table);
                included_has_many.push(hd);
            }
        }
        let has_many_indirect = self.indirect_referring_tables(all_tables);
        
        for (hi, linker) in has_many_indirect {
            if !hi.is_linker_table() && 
            !extension_tables.contains(&hi) &&
            !included_has_many.contains(&hi){
             let ref_table = RefTable{
                            table: hi,
                            column: None,
                            linker_table:Some(linker),
                            is_has_one: false,
                            is_ext: false,
                            is_has_many: true,
                            is_direct: false,
                        };
                    
                referenced_tables.push(ref_table);
                included_has_many.push(hi);
            }
        }
        referenced_tables
    }
    
    ///determine if this table is a linker table
    /// FIXME: make sure that there are 2 different tables referred to it
    fn is_linker_table(&self)->bool{
        let pk = self.primary_columns();
        let fk = self.foreign_columns();
        let uc = self.uninherited_columns();
        if pk.len() == 2 && fk.len() == 2 && uc.len() == 2 {
            return true;
        }
        false
    }
    
    /// determines if the table is owned by some other table
    /// say order_line is owned by orders
    /// which doesn't make sense to be a stand alone window on its own
    /// characteristic: if it has only 1 has_one which is its owning parent table
    /// and no other direct or indirect referring table
    fn is_owned(&self, tables: &Vec<Table>)->bool{
        let has_one = self.referred_tables(tables);
        let has_many = self.referring_tables(tables);
        has_one.len() == 1 && has_many.len() == 0
    }
    
    /// has many indirect
    /// when there is a linker table, bypass the 1:1 relation to the linker table
    /// then create a 1:M relation to the other linked table
    /// Algorithmn: determine whether a table is a linker then get the other linked table
    ///        *get all the referring table
    ///        *for each table that refer to this table
    ///        *if there are only 2 columns and is both primary
    ///            and foreign key at the same time
    ///         and 1 of which refer to the primary column of this table
    ///     * then the other table that is refered is the indirect referring table
    /// returns the table that is indirectly referring to this table and its linker table
    fn indirect_referring_tables<'a>(&self, tables: &'a Vec<Table>)->Vec<(&'a Table, &'a Table)>{
        let mut indirect_referring_tables = Vec::new();
        for (rt, column) in self.referring_tables(tables){
            let rt_pk = rt.primary_columns();
            let rt_fk = rt.foreign_columns();
            let rt_uc = rt.uninherited_columns();
            if rt_pk.len() == 2 && rt_fk.len() == 2 && rt_uc.len() == 2 {
                //println!("{} is a candidate linker table for {}", rt.name, self.name);
                let ref_tables = rt.referred_tables(tables);
                let (_, t0) = ref_tables[0];
                let (_, t1) = ref_tables[1];
                let mut other_table;
                //if self.name == t0.name && self.schema == t0.schema{
                if self == t0 {
                    other_table = t1;
                }
                else{
                    other_table = t0;
                }
                let mut cnt = 0;
                for fk in &rt_fk{
                    if self.is_foreign_column_refer_to_primary_of_this_table(fk){
                        cnt += 1;
                    }
                    if other_table.is_foreign_column_refer_to_primary_of_this_table(fk){
                        cnt += 1;
                    }
                }
                
                if cnt == 2{
                    indirect_referring_tables.push((other_table, rt))
                }
            }
        }
        indirect_referring_tables
    }
    

    
    /// get referring tables, and check if primary columns of these referring table
    /// is the same set of the primary columns of this table
    /// it is just an extension table
    /// [FIXED]~~FIXME:~~ 2 primary 1 foreign should not be included as extension table
    /// case for photo_sizes
    fn extension_tables<'a>(&self, tables: &'a Vec<Table>)->Vec<&'a Table>{
        let mut extension_tables = Vec::new();
        for (rt, _) in self.referring_tables(tables){
            let pkfk = rt.primary_and_foreign_columns();
            let rt_pk = rt.primary_columns();
            //if the referring tables's foreign columns are also its primary columns
            //that refer to the primary columns of this table
            //then that table is just an extension table of this table
            if rt_pk == pkfk && pkfk.len() > 0 {
                //if all fk refer to the primary of this table
                if self.are_these_foreign_column_refer_to_primary_of_this_table(&pkfk){
                    extension_tables.push(rt);
                }
            }
        }
        extension_tables
    }
    
    /// returns only columns that are both primary and foreign
    /// FIXME: don't have to do this if the function getmeta data has merged this.
    fn primary_and_foreign_columns(&self)->Vec<&Column>{
        let mut both = Vec::new();
        let pk = self.primary_columns();
        let fk = self.foreign_columns();
        for f in fk{
            if pk.contains(&f){
                //println!("{}.{} is both primary and foreign", self.name, f.name);
                both.push(f);
            }
        }
        both
    }
    
    fn is_foreign_column_refer_to_primary_of_this_table(&self, fk:&Column)->bool{
        if fk.foreign.is_some(){
            let foreign = fk.foreign.clone().unwrap();
            let table = foreign.table;
            let schema = foreign.schema;
            let column = foreign.column;
            if self.name == table && self.schema == schema && self.is_primary(&column){
                return true;
            }
        }
        false
    }
        
    fn are_these_foreign_column_refer_to_primary_of_this_table(&self, rt_fk:&Vec<&Column>)->bool{
        let mut cnt = 0;
        for fk in rt_fk{
            if self.is_foreign_column_refer_to_primary_of_this_table(fk){
                cnt += 1;
            }
        }
        cnt == rt_fk.len()
    }

}


fn capitalize(str:&str)->String{
     str.chars().take(1)
         .flat_map(char::to_uppercase)
        .chain(str.chars().skip(1))
        .collect()
}

#[test]
fn test_capitalize(){
    assert_eq!(capitalize("hello"), "Hello".to_string());
}
