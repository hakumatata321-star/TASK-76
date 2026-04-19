use leptos::*;
use crate::api::types::MaskedVehicle;

#[component]
pub fn VehicleCard(vehicle: MaskedVehicle) -> impl IntoView {
    view! {
        <div class="card">
            <div style="display: flex; justify-content: space-between; align-items: start;">
                <h3>{vehicle.make.clone()} " " {vehicle.model.clone()} " " {vehicle.trim_level.clone()}</h3>
                <crate::components::status_badge::StatusBadge status=vehicle.status.clone() />
            </div>
            <table style="font-size: 0.875rem;">
                <tr><td style="font-weight: 500;">"VIN"</td><td class="masked">{vehicle.vin.clone()}</td></tr>
                <tr><td style="font-weight: 500;">"Plate"</td><td class="masked">{vehicle.license_plate.clone()}</td></tr>
                <tr><td style="font-weight: 500;">"Mileage"</td><td>{crate::utils::format::format_mileage(vehicle.mileage_miles)}</td></tr>
                <tr><td style="font-weight: 500;">"Fuel/Battery"</td><td>{format!("{:.0}%", vehicle.fuel_or_battery_pct)}</td></tr>
                <tr><td style="font-weight: 500;">"Store"</td><td>{vehicle.store_id.clone()}</td></tr>
            </table>
            {vehicle.insurance_expiry.as_ref().map(|d| view! {
                <p style="font-size: 0.75rem; color: #6b7280; margin-top: 0.5rem;">"Insurance expires: " {d.clone()}</p>
            })}
            {vehicle.maintenance_due.as_ref().map(|d| view! {
                <p style="font-size: 0.75rem; color: #6b7280;">"Maintenance due: " {d.clone()}</p>
            })}
        </div>
    }
}
