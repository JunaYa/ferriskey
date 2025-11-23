#!/bin/bash
FILES=(
"/Users/arjun/Code/yovo/ferriskey/core/src/domain/webhook/services.rs"
"/Users/arjun/Code/yovo/ferriskey/core/src/domain/credential/services.rs"
"/Users/arjun/Code/yovo/ferriskey/core/src/domain/role/services.rs"
"/Users/arjun/Code/yovo/ferriskey/core/src/domain/health/services.rs"
"/Users/arjun/Code/yovo/ferriskey/core/src/domain/seawatch/services.rs"
"/Users/arjun/Code/yovo/ferriskey/core/src/domain/user/services.rs"
"/Users/arjun/Code/yovo/ferriskey/core/src/domain/prompt/services.rs"
"/Users/arjun/Code/yovo/ferriskey/core/src/domain/trident/services.rs"
"/Users/arjun/Code/yovo/ferriskey/core/src/domain/authentication/services/grant_type_service.rs"
"/Users/arjun/Code/yovo/ferriskey/core/src/domain/authentication/services/authenticate.rs"
"/Users/arjun/Code/yovo/ferriskey/core/src/domain/client/services.rs"
"/Users/arjun/Code/yovo/ferriskey/core/src/domain/realm/services.rs"
)

for file in "${FILES[@]}"; do
    if [ -f "$file" ]; then
        if ! grep -q "food_analysis::ports" "$file"; then
            # Find the first use crate::domain line and add after it
            sed -i '' '/^use crate::domain::/a\
    food_analysis::ports::{FoodAnalysisRepository, LLMClient},' "$file"
        fi
    fi
done
