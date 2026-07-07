use eframe::egui;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

const CSV_PATH: &str = "ISCTRACKER.csv";



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectronicComponent {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Part Number")]
    pub part_number: Option<String>,
    #[serde(rename = "Description")]
    pub description: Option<String>,
    #[serde(rename = "Value")]
    pub value: Option<String>,
    #[serde(rename = "Footprint")]
    pub footprint: Option<String>,
    #[serde(rename = "Price")]
    pub price: Option<f64>,
    #[serde(rename = "Stock")]
    pub stock: u32,
    #[serde(rename = "Container")]
    pub container: Option<String>,
    #[serde(rename = "Notes")]
    pub notes: Option<String>,
}

impl ElectronicComponent {
    pub fn new_empty() -> Self {
        Self {
            id: format!("IFS08-{}", Uuid::new_v4()), 
            part_number: None,
            description: None,
            value: None,
            footprint: None,
            price: None,
            stock: 0,
            container: None,
            notes: None,
        }
    }

    pub fn matches_query(&self, query: &str) -> bool {
        let q = query.to_lowercase();
        if q.is_empty() { return true; }

        let fields = [
            self.part_number.as_deref().unwrap_or(""),
            self.description.as_deref().unwrap_or(""),
            self.value.as_deref().unwrap_or(""),
            self.footprint.as_deref().unwrap_or(""),
            self.container.as_deref().unwrap_or(""),
            self.notes.as_deref().unwrap_or(""),
        ];

        fields.iter().any(|f| f.to_lowercase().contains(&q))
            || self.stock.to_string().contains(&q)
            || self.id.to_lowercase().contains(&q)
    }
}

// --- LÓGICA DE ARCHIVOS ---

fn load_inventory() -> Vec<ElectronicComponent> {
    if !Path::new(CSV_PATH).exists() {
        return Vec::new();
    }
    let mut inventory = Vec::new();
    
    if let Ok(mut rdr) = csv::ReaderBuilder::new().delimiter(b';').from_path(CSV_PATH) {
        for result in rdr.deserialize::<HashMap<String, String>>() {
            if let Ok(map) = result {
                let id = map.get("ID")
                    .and_then(|s| {
                        let trimmed = s.trim();
                        if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
                    })
                    .unwrap_or_else(|| format!("IFS08-{}", Uuid::new_v4()));
                
                let get_opt_string = |key: &str| -> Option<String> {
                    map.get(key).cloned().and_then(|s| {
                        let trimmed = s.trim();
                        if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
                    })
                };

                let part_number = get_opt_string("Part Number");
                let description = get_opt_string("Description");
                let value = get_opt_string("Value");
                let footprint = get_opt_string("Footprint");
                let container = get_opt_string("Container");
                let notes = get_opt_string("Notes");
                
                let price = map.get("Price")
                    .and_then(|s| s.trim().replace(',', ".").parse::<f64>().ok());
                    
                let stock = map.get("Stock")
                    .and_then(|s| s.trim().parse::<u32>().ok())
                    .unwrap_or(0);

                inventory.push(ElectronicComponent {
                    id,
                    part_number,
                    description,
                    value,
                    footprint,
                    price,
                    stock,
                    container,
                    notes,
                });
            }
        }
    }
    inventory
}

fn save_inventory(inventory: &[ElectronicComponent]) {
    if let Ok(mut wtr) = csv::WriterBuilder::new().delimiter(b';').from_path(CSV_PATH) {
        for comp in inventory {
            let _ = wtr.serialize(comp);
        }
        let _ = wtr.flush();
    }
}

// --- ESTADO DE LA APLICACIÓN ---

struct IscTrackerApp {
    inventory: Vec<ElectronicComponent>,
    search_query: String,
    show_add_window: bool,
    new_component: ElectronicComponent,
    editing_component: Option<ElectronicComponent>, // NUEVO: Estado para editar
}

impl IscTrackerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let inventory = load_inventory();
        save_inventory(&inventory);

        Self {
            inventory,
            search_query: String::new(),
            show_add_window: false,
            new_component: ElectronicComponent::new_empty(),
            editing_component: None, // Inicializamos vacío
        }
    }
}

// --- UTILIDAD PARA LA UI ---

fn text_edit_option(ui: &mut egui::Ui, label: &str, opt: &mut Option<String>) {
    ui.horizontal(|ui| {
        ui.label(label);
        let mut s = opt.as_deref().unwrap_or("").to_string();
        if ui.text_edit_singleline(&mut s).changed() {
            *opt = if s.trim().is_empty() { None } else { Some(s.trim().to_string()) };
        }
    });
}

// --- INTERFAZ GRÁFICA ---

impl eframe::App for IscTrackerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut needs_save = false;

        // VENTANA MODAL: Añadir Componente
        if self.show_add_window {
            egui::Window::new("Añadir Nuevo Componente")
                .collapsible(false)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.group(|ui| {
                        text_edit_option(ui, "Part Number:", &mut self.new_component.part_number);
                        text_edit_option(ui, "Description:", &mut self.new_component.description);
                        text_edit_option(ui, "Value:", &mut self.new_component.value);
                        text_edit_option(ui, "Footprint:", &mut self.new_component.footprint);
                        text_edit_option(ui, "Container:", &mut self.new_component.container);
                        text_edit_option(ui, "Notes:", &mut self.new_component.notes);
                        
                        ui.horizontal(|ui| {
                            ui.label("Stock:");
                            ui.add(egui::DragValue::new(&mut self.new_component.stock).speed(1));
                        });
                    });

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("💾 Guardar").clicked() {
                            self.inventory.push(self.new_component.clone());
                            needs_save = true;
                            self.show_add_window = false;
                            self.new_component = ElectronicComponent::new_empty();
                        }
                        if ui.button("❌ Cancelar").clicked() {
                            self.show_add_window = false;
                            self.new_component = ElectronicComponent::new_empty();
                        }
                    });
                });
        }

        // VENTANA MODAL: Editar Componente (NUEVA)
        let mut close_edit = false;
        let mut save_edit = false;

        if let Some(edit_comp) = &mut self.editing_component {
            egui::Window::new("Editar Componente")
                .collapsible(false)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.group(|ui| {
                        text_edit_option(ui, "Part Number:", &mut edit_comp.part_number);
                        text_edit_option(ui, "Description:", &mut edit_comp.description);
                        text_edit_option(ui, "Value:", &mut edit_comp.value);
                        text_edit_option(ui, "Footprint:", &mut edit_comp.footprint);
                        text_edit_option(ui, "Container:", &mut edit_comp.container);
                        text_edit_option(ui, "Notes:", &mut edit_comp.notes);
                        
                        ui.horizontal(|ui| {
                            ui.label("Stock:");
                            ui.add(egui::DragValue::new(&mut edit_comp.stock).speed(1));
                        });
                    });

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Guardar Cambios").clicked() {
                            save_edit = true;
                        }
                        if ui.button("Cancelar").clicked() {
                            close_edit = true;
                        }
                    });
                });
        }

        // Procesar las acciones de edición fuera de la ventana
        if save_edit {
            if let Some(edited_comp) = self.editing_component.take() {
                // Buscamos el componente original por su ID y lo sustituimos por el editado
                if let Some(pos) = self.inventory.iter().position(|c| c.id == edited_comp.id) {
                    self.inventory[pos] = edited_comp;
                    needs_save = true;
                }
            }
        }
        if close_edit {
            self.editing_component = None; // Cerramos la ventana simplemente reseteando el estado
        }


        // PANEL SUPERIOR: Búsqueda y Botón
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.heading("Inventario ISC");
                ui.separator();
                ui.label("🔍 Buscar:");
                ui.text_edit_singleline(&mut self.search_query);
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("➕ Añadir Componente").clicked() {
                        self.show_add_window = true;
                    }
                });
            });
            ui.add_space(10.0);
        });

        // PANEL CENTRAL: Lista de Componentes
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut id_to_delete = None;

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    for comp in &mut self.inventory {
                        if comp.matches_query(&self.search_query) {
                            let title = format!(
                                "{} | {} | {}", 
                                comp.part_number.as_deref().unwrap_or("N/A"), 
                                comp.description.as_deref().unwrap_or("Sin descripción"),
                                comp.value.as_deref().unwrap_or("-")
                            );

                            egui::CollapsingHeader::new(title)
                                .id_source(&comp.id)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("📦 Stock Actual: {}", comp.stock));
                                        ui.label(format!("| Footprint: {}", comp.footprint.as_deref().unwrap_or("-")));
                                        ui.label(format!("| Caja: {}", comp.container.as_deref().unwrap_or("-")));
                                        ui.label(format!("| ID: {}", comp.id));
                                    });

                                    if let Some(notes) = &comp.notes {
                                        ui.label(format!("📝 Notas: {}", notes));
                                    }

                                    ui.add_space(5.0);
                                    
                                    ui.horizontal(|ui| {
                                        if ui.button("➕ Añadir Cantidad").clicked() {
                                            comp.stock += 1;
                                            needs_save = true;
                                        }
                                        if ui.button("➖ Reducir Cantidad").clicked() {
                                            if comp.stock > 0 {
                                                comp.stock -= 1;
                                                needs_save = true;
                                            }
                                        }
                                        ui.separator();
                                        // NUEVO BOTÓN: EDITAR
                                        if ui.button("Editar").clicked() {
                                            self.editing_component = Some(comp.clone());
                                        }
                                        ui.separator();
                                        if ui.button("🗑 Eliminar Componente").clicked() {
                                            id_to_delete = Some(comp.id.clone());
                                        }
                                    });
                                });
                            ui.separator();
                        }
                    }
                });

            if let Some(id) = id_to_delete {
                self.inventory.retain(|c| c.id != id);
                needs_save = true;
            }
        });

        if needs_save {
            save_inventory(&self.inventory);
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 600.0])
            .with_title("ISC Tracker"),
        ..Default::default()
    };

    eframe::run_native(
        "ISC Tracker",
        options,
        Box::new(|cc| Box::new(IscTrackerApp::new(cc))),
    )
}