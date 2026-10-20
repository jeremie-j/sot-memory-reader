use std::{collections::HashMap, ffi::c_void};

use crate::{
    core::reader::{read_array, read_array_sized, read_pointer, ActorInfo},
    services::sdk::sdk_service,
};

pub struct EmmissaryTables {
    pub athena_emissary_table: Option<ActorInfo>,
    pub reaper_emissary_table: Option<ActorInfo>,
    pub sovereign_emissary_table: Option<ActorInfo>,
    pub merchant_alliance_emissary_table: Option<ActorInfo>,
    pub gold_hoarders_emissary_table: Option<ActorInfo>,
    pub order_of_souls_emissary_table: Option<ActorInfo>,
}

impl EmmissaryTables {
    pub fn get_mapping(&self) -> Vec<(&'static str, &Option<ActorInfo>)> {
        vec![
            (
                "BP_EmissaryTable_GoldHoarders_01",
                &self.gold_hoarders_emissary_table,
            ),
            (
                "BP_EmissaryTable_MerchantAlliance_01",
                &self.merchant_alliance_emissary_table,
            ),
            (
                "BP_EmissaryTable_OrderOfSouls_01",
                &self.gold_hoarders_emissary_table,
            ),
            (
                "BP_EmissaryTable_Sov_01_a_C",
                &self.sovereign_emissary_table,
            ),
            (
                "BP_FactionEmissaryTable_Reapers2",
                &self.reaper_emissary_table,
            ),
            (
                "BP_FactionEmissaryTable_Athena",
                &self.athena_emissary_table,
            ),
        ]
    }
}

impl EmmissaryTables {
    fn new() -> Self {
        Self {
            athena_emissary_table: None,
            reaper_emissary_table: None,
            sovereign_emissary_table: None,
            merchant_alliance_emissary_table: None,
            gold_hoarders_emissary_table: None,
            order_of_souls_emissary_table: None,
        }
    }

    // fn update(&self) {
    //     for (table_actor, emissary_label) in vec![
    //         (&self.gold_hoarders_emissary_table, "Gold Hoarders : "),
    //         (&self.merchant_alliance_emissary_table, "Merchants : "),
    //         (&self.order_of_souls_emissary_table, "Order of souls : "),
    //         (&self.sovereign_emissary_table, "Sovereign : "),
    //         (&self.reaper_emissary_table, "Reaper : "),
    //         (&self.athena_emissary_table, "Athena : "),
    //     ] {
    //         if table_actor.is_none() {
    //             continue;
    //         }
    //         let table_actor_info = table_actor.clone().unwrap();
    //         let u_object = read_pointer(table_actor_info.base_address as *mut UObject).unwrap();

    //         let class_ = read_pointer(u_object.u_class).unwrap();
    //         let class_name = read_gname(class_.name.index).unwrap();

    //         let emissary_ship_affiliation_tracker_offset =
    //             sdk_service().get_offset(&format!("{}.EmissaryShipAffiliationTracker", class_name));

    //         let emissary_count_offset =
    //             sdk_service().get_offset("EmissaryShipAffiliationTrackerComponent.EmissaryCount");

    //         let emmisary_count = find_dma_addy::<u32>(
    //             table_actor_info.base_address,
    //             vec![
    //                 emissary_ship_affiliation_tracker_offset,
    //                 emissary_count_offset,
    //             ],
    //         )
    //         .unwrap();

    //         println!("{} {}", emissary_label, emmisary_count);
    //     }
    // }
}

pub struct CrewService {
    actor: ActorInfo,
    crews: HashMap<Guid, u32>,
    my_crew_id: Option<Guid>,
    total_players: u32,
}

#[allow(non_snake_case)]
#[derive(Eq, Hash, PartialEq)]
pub struct Guid {
    pub A: u32,
    pub B: u32,
    pub C: u32,
    pub D: u32,
}

impl CrewService {
    pub fn new(actor: ActorInfo) -> Self {
        Self {
            actor: actor,
            crews: HashMap::new(),
            my_crew_id: None,
            total_players: 0,
        }
    }

    fn get_crews(&self) -> HashMap<Guid, u32> {
        let crew_array = read_array_sized(
            &self.actor.base_address + sdk_service().get_offset("CrewService.Crews") as usize,
            sdk_service().get_class_or_struct_size("Crew") as usize,
        );

        let mut crews_hasmap: HashMap<Guid, u32> = HashMap::new();

        let crew_guid_offset = sdk_service().get_offset("Crew.CrewId");
        let player_array_offset = sdk_service().get_offset("Crew.Players");
        for crew_actor_pointer in crew_array.iter() {
            let crew_base = crew_actor_pointer.item_pointer as usize;
            let crew_guid =
                read_pointer((crew_base + crew_guid_offset as usize) as *mut Guid).unwrap();

            let crew_player_array = read_array::<c_void>(crew_base + player_array_offset as usize);
            crews_hasmap.insert(crew_guid, crew_player_array.count);
        }
        crews_hasmap
    }

    fn get_total_players(&self) -> u32 {
        self.crews.values().sum()
    }

    fn is_valid(&self) -> bool {
        read_pointer(self.actor.base_address as *mut c_void).is_err()
    }

    pub fn update(&mut self) {
        // if !self.is_valid() {
        //     println!("CrewService is not valid");
        //     return;
        // }
        self.crews = self.get_crews();
        self.total_players = self.get_total_players()
    }

    pub fn print_crews(&self) {
        for (index, crew_player_count) in self.crews.values().enumerate() {
            println!("Crew {}: {} players", index, crew_player_count);
        }
        println!("Total players: {}", self.total_players);
    }
}

pub struct World {
    pub events: Vec<ActorInfo>,
    pub crew_service: Option<CrewService>,
    pub emissary_tables: EmmissaryTables,
}

impl World {
    pub fn new() -> Self {
        Self {
            events: vec![],
            crew_service: None,
            emissary_tables: EmmissaryTables::new(),
        }
    }
}
