#pragma once
#include <string>
#include <vector>
#include <map>
#include <memory>

struct Recommendation {
    std::string action;                        // e.g. "Upsell Premium Bundle", "Apply 10% discount"
    double confidence = 0.0;                   // 0.0–1.0
    std::string rationale;                     // human-readable explanation
    std::vector<std::pair<std::string, double>> feature_contributions;  // top factors
};

class RecommendationEngine {
public:
    static std::unique_ptr<RecommendationEngine> create(const std::string& config_path = "config.json");

    virtual ~RecommendationEngine() = default;

    virtual Recommendation recommend(
        const std::string& context_type,                    // e.g. "product_upsell", "pricing_adjust", "inventory_action"
        const std::map<std::string, double>& numeric_features,
        const std::map<std::string, std::string>& categorical_features = {}
    ) = 0;

    virtual void reload_config() = 0;  // hot reload if needed
};
