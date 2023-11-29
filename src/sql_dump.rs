use sqlite;
use regex;
use regex::Regex;

use std::io;
use std::path::{Path, PathBuf};
use std::fmt::Write;

use crate::data::{DataSource, EntryIndex, EntryInfo, TileID};
use crate::archive_data::{walk_entry_list};

pub struct SqlWriter<T: DataSource> {
    data_source: T,
    path: PathBuf,
}

impl<T: DataSource> SqlWriter<T> {
    pub fn new(
        data_source: T,
        path: impl AsRef<Path>,
    ) -> Self {
        Self {
            data_source: data_source,
            path: path.as_ref().to_owned()
        }
    }

    fn write_proc(&mut self, connection: &sqlite::Connection, processor: &regex::Captures<'_>) -> Option<String> {
        let mut table = String::from("");
        write!(&mut table, "Node_{node}_{processor}_{proc_id}",
               node=&processor["node"],
               processor=&processor["proc_type"],
               proc_id=&processor["proc_id"]).unwrap();

        let mut query = String::from("");

        write!(&mut query, "
CREATE TABLE IF NOT EXISTS {table} (
task TEXT,
task_full_name TEXT,
variant TEXT,
operation INTEGER,
interval_start REAL,
interval_stop REAL,
interval_duration REAL,
running_start REAL,
running_stop REAL,
running_duration REAL,
initiation TEXT,
initiation_full_name TEXT,
initiation_variant TEXT,
initiation_operation INTEGER,
instances TEXT,
provenance TEXT
);", table=table).unwrap();

        connection.execute(query).unwrap();

        query = String::from("");
        write!(&mut query, "
INSERT INTO {table} VALUES (@task, @task_full_name, @variant, @operation, @interval_start, @interval_stop, @interval_duration, @running_start, @running_stop, @running_duration, @initiation, @initiation_full_name, @initiation_variant, @initiation_operation, @instances, @provenance);", table=table).unwrap();

        Some(query)
    }

    fn write_util(&mut self, connection: &sqlite::Connection, utility: &regex::Captures<'_>) -> Option<String> {
        let mut table = String::from("");
        write!(&mut table, "Node_{node}_Utility_{proc_id}",
               node=&utility["node"], proc_id=&utility["proc_id"]).unwrap();

        let mut query = String::from("");
        write!(&mut query, "
CREATE TABLE IF NOT EXISTS {table} (
task TEXT,
task_full_name TEXT,
interval_start REAL,
interval_stop REAL,
interval_duration REAL,
running_start REAL,
running_stop REAL,
running_duration REAL,
initiation TEXT,
initiation_full_name TEXT,
initiation_variant TEXT,
initiation_operation INTEGER
);", table=table).unwrap();

        connection.execute(query).unwrap();

        query = String::from("");
        write!(&mut query, "
INSERT INTO {table} VALUES (@task, @task_full_name, @interval_start, @interval_stop, @interval_duration, @running_start, @running_stop, @running_duration, @initiation, @initiation_full_name, @initiation_variant, @initiation_operation);", table=table).unwrap();

        Some(query)
    }

    fn write_mem(&mut self, connection: &sqlite::Connection, memory: &regex::Captures<'_>) -> Option<String> {
        let mut table = String::from("");
        write!(&mut table, "Node_{node}_{memory}_{mem_id}",
               node=&memory["node"], memory=&memory["mem_type"], mem_id=&memory["mem_id"]).unwrap();

        let mut query = String::from("");
        write!(&mut query, "
CREATE TABLE IF NOT EXISTS {table} (
instance TEXT,
interval_start REAL,
interval_stop REAL,
interval_duration REAL,
index_space TEXT,
field_space TEXT,
fields TEXT,
layout TEXT,
size REAL,
initiation TEXT,
initiation_full_name TEXT,
initiation_variant TEXT,
initiation_operation INTEGER,
provenance TEXT
);",
               table=table).unwrap();

        connection.execute(query).unwrap();

        query = String::from("");
        write!(&mut query, "
INSERT INTO {table} VALUES (@instance, @interval_start, @interval_stop, @interval_duration, @index_space, @field_space, @fields, @layout, @size, @initation, @initiation_full_name, @initiation_variant, @initiation_operation, @provenance);",
               table=table).unwrap();

        Some(query)
    }

    fn write_chan(&mut self, connection: &sqlite::Connection, channel: &regex::Captures<'_>) -> Option<String> {
        let mut table = String::from("");
        write!(&mut table, "Node_{src_node}_{src_mem}_{src_mem_id}_to_Node_{dst_node}_{dst_mem}_{dst_mem_id}",
               src_node=&channel["src_node"],
               src_mem=&channel["src_mem"],
               src_mem_id=&channel["src_mem_id"],
               dst_node=&channel["dst_node"],
               dst_mem=&channel["dst_mem"],
               dst_mem_id=&channel["dst_mem_id"]).unwrap();

        let mut query = String::from("");
        write!(&mut query, "
CREATE TABLE IF NOT EXISTS {table} (
interval_start REAL,
interval_stop REAL,
interval_duration REAL,
requirements TEXT,
source_inst TEXT,
source_fields TEXT,
dest_inst TEXT,
dest_fields TEXT,
hops INT,
size REAL,
initiation TEXT,
initiation_full_name TEXT,
initiation_variant TEXT,
initiation_operation INTEGER,
provenance TEXT
);",
               table=table).unwrap();

        connection.execute(query).unwrap();

        query = String::from("");
        write!(&mut query, "
INSERT INTO {table} VALUES (@interval_start, @interval_stop, @interval_duration, @requirements, @source_inst, @source_fields, @dest_inst, @dest_fields, @hops, @size, @initiation, @initiation_full_name, @initiation_variant, @initiation_operation, @provenance);
",
               table=table).unwrap();

        Some(query)
    }

    pub fn write(mut self) -> io::Result<()> {
        let connection = sqlite::open(&self.path).unwrap();

        let info = self.data_source.fetch_info();

        let entry_ids = walk_entry_list(&info.entry_info);

        let proc = Regex::new(r"Node (?P<node>\d+) (?P<proc_type>(GPU|CPU|IO|ProcGroup|ProcSet|OpenMP|Python)) (?P<proc_id>\d+)").unwrap();

        let util = Regex::new(r"Node (?P<node>\d+) Utility (?P<proc_id>\d+)").unwrap();

        let fill = Regex::new(r"Fill Node (?P<node>\d+) (?P<mem_type>(Global|System|Registered|Socket|ZeroCopy|Framebuffer|Disk|HDF5|File|L3Cache|L2Cache|L1Cache|GPUManaged|GPUDynamic)) (?P<mem_id>\d+)").unwrap();

        let mem = Regex::new(r"Node (?P<node>\d+) (?P<mem_type>(Global|System|Registered|Socket|ZeroCopy|Framebuffer|Disk|HDF5|File|L3Cache|L2Cache|L1Cache|GPUManaged|GPUDynamic)) (?P<mem_id>\d+)").unwrap();

        let chan = Regex::new(r"Node (?P<src_node>\d+) (?P<src_mem>(Global|System|Registered|Socket|ZeroCopy|Framebuffer|Disk|HDF5|File|L3Cache|L2Cache|L1Cache|GPUManaged|GPUDynamic)) (?P<src_mem_id>\d+) to Node (?P<dst_node>\d+) (?P<dst_mem>(Global|System|Registered|Socket|ZeroCopy|Framebuffer|Disk|HDF5|File|L3Cache|L2Cache|L1Cache|GPUManaged|GPUDynamic)) (?P<dst_mem_id>\d+)").unwrap();

        // let interval = Regex::new();
        let task = Regex::new(r"(?P<name>(\w+)) \[(?P<variant>(\w+))\] \<(?P<operation>(\d+))\>").unwrap();

        for entry_id in &entry_ids {

            let entry_info = info.entry_info.get(entry_id).unwrap();
            let name : Option<&str>;

            match entry_info {
                EntryInfo::Panel{ long_name, ..} => { name = Some(long_name); }
                EntryInfo::Slot{ long_name, .. } => { name = Some(long_name); }
                _ => { continue }
            }

            println!("ENTRY: {}", name.unwrap());

            let mut query : Option<String> = None;
            let mut typ : Option<&str> = None;

            let processor = proc.captures(name.unwrap());

            match processor {
                Some(processor) => { query = self.write_proc(&connection, &processor); typ = Some("processor") },
                None => {}
            }

            let utility = util.captures(name.unwrap());

            match utility {
                Some(utility) => { query = self.write_util(&connection, &utility); typ = Some("utility") },
                None => {}
            }

            let memory = mem.captures(name.unwrap());

            match memory {
                Some(memory) => { query = self.write_mem(&connection, &memory); typ = Some("memory") },
                None => {}
            }

            let channel = chan.captures(name.unwrap());

            match channel {
                Some(channel) => { query = self.write_chan(&connection, &channel); typ = Some("channel") },
                None => {}
            }

            connection.execute("BEGIN TRANSACTION").unwrap();

            let mut statement = connection.prepare(query.unwrap()).unwrap();

            match entry_id.last_index().unwrap() {
                EntryIndex::Slot(..) => {
                    let slot_meta_tile = self.data_source.fetch_slot_meta_tile(
                        entry_id, TileID(info.interval), true);
                    for row in &slot_meta_tile.data.items {
                        for item in row.iter() {
                            println!("\t{}", item.title);
                            match typ.unwrap() {
                                "processor" => {
                                    statement.bind(("@task_full_name", item.title.as_str())).unwrap();
                                    let t = task.captures(item.title.as_str());
                                    match t {
                                        Some(t) => { statement.bind(("@task", &t["name"])).unwrap();
                                                     statement.bind(("@variant", &t["variant"])).unwrap();
                                        },
                                        None => {}
                                    }
                                },
                                "utility" => { statement.bind(("@task_full_name", item.title.as_str())).unwrap() },
                                "memory" => { statement.bind(("@instance", item.title.as_str())).unwrap() },
                                // _ => { panic!("Unknown type") }
                                _ => {}
                            }

                            for (field_id, field) in &item.fields {
                                let name = info.field_schema.get_name(*field_id).unwrap();
                                let mut fstr = String::from("");
                                write!(&mut fstr, "{}", field).unwrap();
                                match name {
                                    "Interval" => { },
                                    "Running" => { },
                                    "Operation" => { statement.bind(("@operation", fstr.as_str())).unwrap(); },
                                    "Initiation" => { statement.bind(("@initiation_full_name", fstr.as_str())).unwrap(); },
                                    "Instances" => { statement.bind(("@instances", fstr.as_str())).unwrap(); },
                                    "Provenance" => { statement.bind(("@provenance", fstr.as_str())).unwrap(); },
                                    "Index Space" => { statement.bind(("@index_space", fstr.as_str())).unwrap(); },
                                    "Field Space" => { statement.bind(("@field_space", fstr.as_str())).unwrap(); },
                                    "Fields" => { statement.bind(("@fields", fstr.as_str())).unwrap(); },
                                    "Layout" => { statement.bind(("@layout", fstr.as_str())).unwrap(); },
                                    // "Size" => { statement.bind(("@size", fstr.as_str())).unwrap(); },
                                    // _ => { panic!("Unknown field")}
                                    _ => { }
                                }
                                println!("\t\tName: {} Value: {}", name, field)
                            }
                            statement.next().unwrap();
                            statement.reset().unwrap();
                        }
                    }
                }
                _ => {}
            }

            connection.execute("END TRANSACTION").unwrap();

        }
        Ok(())
    }
}
