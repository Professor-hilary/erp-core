#include "recommendation_engine.h"

#include <dlib/random_forest.h>  // random_forest_regression_trainer etc.
#include <dlib/svm_threaded.h>   // for trainers & kernels
#include <spdlog/sinks/basic_file_sink.h>
#include <spdlog/sinks/stdout_color_sinks.h>
#include <spdlog/spdlog.h>

#include <chrono>
#include <format>
#include <fstream>
#include <nlohmann/json.hpp>

using json = nlohmann::json;

// Base strategy interface
class RecommendationStrategy {
   public:
    virtual ~RecommendationStrategy() = default;
    virtual Recommendation execute(
        const std::string& context,
        const std::map<std::string, double>& num_feats,
        const std::map<std::string, std::string>& cat_feats) = 0;
};

// dlib-based ML strategy (Random Forest regression → score → map to action)
class DlibRFStrategy : public RecommendationStrategy {
    dlib::random_forest_regression_function<dlib::dense_feature_extractor>
        model;
    std::vector<std::string> feature_order;  // fixed order for input vector

   public:
    DlibRFStrategy() {
        // In v1: dummy / simple model (in production: load serialized model)
        spdlog::info(
            "dlib Random Forest strategy initialized (regression scoring)");
        // Example: assume 5 features in fixed order
        feature_order = {"customer_ltv", "margin_potential", "recency_score",
                         "stock_level", "demand_trend"};
    }

    Recommendation execute(const std::string& context,
                           const std::map<std::string, double>& num_feats,
                           const std::map<std::string, std::string>&) override {
        dlib::matrix<double, 0, 1> sample(feature_order.size());

        // Map input to fixed vector (missing = 0 or mean imputation in real
        // impl)
        for (size_t i = 0; i < feature_order.size(); ++i) {
            auto it = num_feats.find(feature_order[i]);
            sample(i) = (it != num_feats.end()) ? it->second : 0.0;
        }

        // Predict score (0–1 range after normalization)
        double raw_score = model(sample);
        double conf = std::clamp((raw_score + 1.0) / 2.0, 0.0,
                                 1.0);  // dummy sigmoid-like

        std::string action;
        if (conf > 0.70) {
            action = context + " → High Priority Upsell / Increase Price";
        } else if (conf > 0.45) {
            action = context + " → Moderate Recommendation / Conditional Offer";
        } else {
            action = context + " → No Action / Monitor";
        }

        // Dummy feature importance (in real: use OOB permutation importance
        // from dlib)
        std::vector<std::pair<std::string, double>> contribs;
        for (const auto& f : feature_order) {
            contribs.emplace_back(f,
                                  0.15 + (rand() % 30) / 100.0);  // placeholder
        }

        return {action, conf, "dlib RF regression score → mapped action",
                contribs};
    }
};

// Placeholder for other strategies (expand later)
class WeightedScoringStrategy : public RecommendationStrategy { /* ... */
};
class ControlFlowStrategy : public RecommendationStrategy { /* ... */
};
// etc.

// Main engine implementation
class RecommendationEngineImpl : public RecommendationEngine {
    json config;
    std::unique_ptr<RecommendationStrategy> strategy;
    std::shared_ptr<spdlog::logger> logger;

   public:
    RecommendationEngineImpl(const std::string& config_path) {
        std::ifstream f(config_path);
        if (!f) throw std::runtime_error("Config not found: " + config_path);
        config = json::parse(f);

        // Setup logging
        auto console = spdlog::stdout_color_mt("console");
        logger = spdlog::basic_logger_mt("audit", "erp_recommendations.log");
        spdlog::set_default_logger(console);

        // Select strategy
        std::string strat_name = config.value("strategy", "dlib_rf");
        if (strat_name == "dlib_rf") {
            strategy = std::make_unique<DlibRFStrategy>();
        } else {
            // fallback or throw
            strategy = std::make_unique<DlibRFStrategy>();
        }

        logger->info("Engine started with strategy: {}", strat_name);
    }

    Recommendation recommend(
        const std::string& context_type,
        const std::map<std::string, double>& numeric_features,
        const std::map<std::string, std::string>& categorical_features)
        override {
        auto start = std::chrono::steady_clock::now();

        auto rec = strategy->execute(context_type, numeric_features,
                                     categorical_features);

        // Audit
        json log_entry = {
            {"timestamp",
             std::chrono::system_clock::now().time_since_epoch().count()},
            {"context", context_type},
            {"action", rec.action},
            {"confidence", rec.confidence},
            {"rationale", rec.rationale}};
        logger->info(log_entry.dump());

        return rec;
    }

    void reload_config() override {
        // TODO: reload config.json and recreate strategy if needed
    }
};

std::unique_ptr<RecommendationEngine> RecommendationEngine::create(
    const std::string& path) {
    return std::make_unique<RecommendationEngineImpl>(path);
}
