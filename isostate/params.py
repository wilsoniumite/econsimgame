"""Default parameters. Everything tunable lives here as a plain dict."""


def default_params(**overrides):
    p = dict(
        seed=7,
        W=120,
        H=84,
        n_towns=6,
        pop0=135_000,
        T_max=48,
        F_max=20_000,
        warmup_years=10,
        # people
        participation=0.5,
        growth_slope=0.03,
        growth_base=-0.004,
        growth_cap=0.011,
        migration_rate=0.05,
        migration_dist=60.0,
        # transport
        freight=0.015,
        handling=0.03,
        trade_k=0.6,
        trade_cap=0.3,
        road_thresholds=(1500.0, 9000.0, 40000.0),
        # markets
        logit_theta=0.25,
        heat_theta=0.2,
        price_speed=0.03,
        price_flow=0.08,
        labor_adjust_ex=0.15,
        labor_adjust_firm=0.35,
        price_target_months=2.0,
        price_floor=0.05,
        decay=(0.03, 0.01, 0.005, 0.002, 0.002, 0.002, 0.003, 0.01, 0.002, 0.002, 0.003, 0.005, 0.002, 0.003, 0.002, 0.5),
        price_ceiling=40.0,
        # households
        cons_income=0.9,
        cons_wealth=0.035,
        lord_share=0.01,
        lord_cons_income=0.75,
        lord_cons_wealth=0.04,
        urban_rent_rate=0.12,
        tools_boost=0.5,
        tools_scale=0.01,
        # money and ownership
        money_growth=0.02,
        rent_regime="landlords",   # or "commons": land rent paid out as an equal dividend
        firm_div_rate=0.4,
        firm_buffer_months=4.0,
        merchant_spread=0.06,
        merch_div_rate=0.15,
        merch_buffer_months=2.0,
        # firms
        hurdle=0.12,
        invest_frac_hh=0.08,
        invest_frac_lord=0.25,
        util_target_margin=0.03,
        util_band=0.04,
        depreciation=0.04,
        entry_K_frac=0.4,
        cluster_K_mult=6.0,
        footprint_K_mult=4.0,
        agglo_same=0.08,
        candidates=32,
        # mines
        pump_input=0.0004,
        pump_boost=2.0,
        wagon_input=0.002,
        wagon_boost=0.6,
        husbandry_boost=0.6,
        farm_mach_boost=0.8,
        farm_mach_input=0.0004,
        # land market
        land_theta=3.0,
        land_adjust=0.2,
        land_fixed_cost=0.15,
        expect_rate=0.3,
        # forests
        forest_capacity=16000.0,
        forest_regrow=0.12,
        clearing_timber=3000.0,
        # tech
        discovery_rate=0.18,
        diffusion_rate=0.3,
        diffusion_dist=35.0,
        learn_cap=4.0,
        # railways
        rail_rails_per_cell=40.0,
        rail_timber_per_cell=30.0,
        rail_stone_per_cell=30.0,
        rail_hurdle=0.06,
        rail_induced=2.5,
        rail_max_projects=6,
        rail_know=0.25,
        spur_min_flow=20000.0,
        # towns
        found_activity=600.0,
        found_distance=10.0,
        urban_density0=500.0,
        urban_density_growth=14.0,
        # detectors
        crossing_lo=0.45,
        crossing_hi=0.55,
    )
    p.update(overrides)
    return p
