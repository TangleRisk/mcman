use anyhow::{Result, anyhow};

use crate::{model::{Downloadable, SoftwareType, World}, util::SelectItem};

use super::{App, AddonType};

impl App {
    pub fn save_changes(&self) -> Result<()> {
        self.server.save()?;

        if let Some(nw) = &self.network {
            nw.save()?;
        }

        Ok(())
    }

    pub async fn refresh_markdown(&self) -> Result<()> {
        if self.server.markdown.auto_update {
            self.markdown().update_files().await
        } else {
            Ok(())
        }
    }

    pub fn add_addon_inferred(&mut self, addon: &Downloadable) -> Result<()> {
        let addon_type = match self.server.jar.get_software_type() {
            SoftwareType::Modded => AddonType::Mod,
            SoftwareType::Normal | SoftwareType::Proxy => AddonType::Plugin,
            SoftwareType::Unknown => self.select("Import as?", &[
                SelectItem(AddonType::Mod, "Mod".to_owned()),
                SelectItem(AddonType::Plugin, "Plugin".to_owned()),
            ])?
        };

        self.add_addon(addon_type, addon)
    }

    pub fn add_addon(&mut self, addon_type: AddonType, addon: &Downloadable) -> Result<()> {
        let existing = match addon_type {
            AddonType::Plugin => self.server.plugins.iter(),
            AddonType::Mod => self.server.mods.iter(),
        }.filter(|o| addon.is_same_as(o)).collect::<Vec<_>>();

        if !existing.is_empty() && self.confirm(
            &format!("{} matching {addon_type}(s) found in server.toml, continue?", existing.len())
        )? {
            return Ok(());
        }

        match addon_type {
            AddonType::Plugin => &mut self.server.plugins,
            AddonType::Mod => &mut self.server.mods,
        }.push(addon.clone());

        self.success(format!("Added {addon_type}: {}", addon.to_short_string()))?;

        Ok(())
    }

    pub fn add_datapack(&mut self, dp: &Downloadable) -> Result<()> {
        let selected_world_name = if self.server.worlds.is_empty() {
            "+".to_owned()
        } else {
            let mut items: Vec<SelectItem<String>> = self
                .server
                .worlds
                .keys()
                .map(|k| SelectItem(k.clone(), k.clone()))
                .collect();
    
            items.push(SelectItem("+".to_owned(), "+ New world entry".to_owned()));
    
            self.select("Add datapack to...", &items)?
        };
    
        let world_name = if selected_world_name == "+" {
            self.prompt_string_default("World Name", "world")?
        } else {
            selected_world_name
        };
    
        if !self.server.worlds.contains_key(&world_name) {
            self.server.worlds.insert(world_name.clone(), World::default());
        }
    
        self.add_datapack_to(&world_name, dp)?;

        Ok(())
    }

    pub fn add_datapack_to(&mut self, world: &str, dp: &Downloadable) -> Result<()> {
        self
            .server
            .worlds
            .get_mut(world)
            .ok_or(anyhow!("World entry did not exist"))?
            .datapacks
            .push(dp.clone());

        self.success(format!("Added datapack {} to {world}", dp.to_short_string()))?;

        Ok(())
    }
}