# Flutter App API 接入文档

## 概述

本文档说明如何在 Flutter 应用中接入 FerrisKey Food Scanner API。系统支持两种认证模式：

1. **设备认证模式**（推荐用于新设备用户）：
   - **仅需 `X-Device-Id` 头**
   - **完全不需要 `Authorization` 头**
   - 无需登录，系统会自动创建匿名用户和设备配置

2. **标准认证模式**（已登录用户）：
   - 需要 `Authorization: Bearer <token>` 头
   - 可选：同时提供 `X-Device-Id` 头（用于设备关联）

**重要说明**：对于新设备用户，使用设备认证模式时，**可以完全省略 `Authorization` 头**，只需提供 `X-Device-Id` 即可。

## 快速开始

### 1. 获取设备 ID

在 Flutter 中，可以使用 `device_info_plus` 包获取设备唯一标识：

```yaml
# pubspec.yaml
name: food_scanner_app
description: Food Scanner Flutter App

dependencies:
  flutter:
    sdk: flutter

  # 网络请求
  http: ^1.2.0

  # 设备信息
  device_info_plus: ^10.1.0

  # UUID 生成
  uuid: ^4.3.0

  # 本地存储
  shared_preferences: ^2.2.0

  # 状态管理（可选，根据需要选择）
  provider: ^6.1.0
  # 或
  # riverpod: ^2.4.0
  # 或
  # flutter_bloc: ^8.1.0
```

```dart
import 'package:device_info_plus/device_info_plus.dart';
import 'package:uuid/uuid.dart';

class DeviceIdService {
  static final DeviceInfoPlugin _deviceInfo = DeviceInfoPlugin();
  static final Uuid _uuid = Uuid();

  /// 获取或生成设备 ID
  /// 优先使用设备硬件 ID，如果无法获取则使用本地存储的 UUID
  static Future<String> getDeviceId() async {
    try {
      if (Platform.isAndroid) {
        final androidInfo = await _deviceInfo.androidInfo;
        return androidInfo.id; // Android ID
      } else if (Platform.isIOS) {
        final iosInfo = await _deviceInfo.iosInfo;
        return iosInfo.identifierForVendor ?? await _generateLocalUuid();
      }
    } catch (e) {
      print('Error getting device ID: $e');
    }
    return await _generateLocalUuid();
  }

  /// 生成本地 UUID 并持久化存储
  static Future<String> _generateLocalUuid() async {
    // 使用 SharedPreferences 存储，确保每次启动使用相同的 UUID
    final prefs = await SharedPreferences.getInstance();
    String? storedUuid = prefs.getString('device_uuid');
    if (storedUuid == null) {
      storedUuid = _uuid.v4();
      await prefs.setString('device_uuid', storedUuid);
    }
    return storedUuid;
  }
}
```

### 2. 创建 API 客户端

```dart
import 'package:http/http.dart' as http;
import 'dart:convert';

class FoodScannerApiClient {
  final String baseUrl;
  final String realmName;
  final String deviceId;
  String? accessToken; // 可选：如果用户已登录

  FoodScannerApiClient({
    required this.baseUrl,
    required this.realmName,
    required this.deviceId,
    this.accessToken,
  });

  /// 获取请求头
  ///
  /// 设备认证模式：仅需 X-Device-Id，无需 Authorization
  /// 标准认证模式：同时提供 X-Device-Id 和 Authorization
  Map<String, String> get headers {
    final headers = {
      'Content-Type': 'application/json',
      'X-Device-Id': deviceId, // 必需：设备标识
    };

    // 可选：如果用户已登录，添加 Authorization 头
    // 注意：对于新设备用户，可以完全省略此头
    if (accessToken != null) {
      headers['Authorization'] = 'Bearer $accessToken';
    }

    return headers;
  }

  /// 发送 POST 请求
  Future<Map<String, dynamic>> post(
    String path,
    Map<String, dynamic> body,
  ) async {
    final url = Uri.parse('$baseUrl/realms/$realmName$path');
    final response = await http.post(
      url,
      headers: headers,
      body: jsonEncode(body),
    );

    if (response.statusCode >= 200 && response.statusCode < 300) {
      return jsonDecode(response.body) as Map<String, dynamic>;
    } else {
      throw ApiException(
        statusCode: response.statusCode,
        message: response.body,
      );
    }
  }

  /// 发送 GET 请求
  Future<Map<String, dynamic>> get(
    String path, {
    Map<String, String>? queryParams,
  }) async {
    var url = Uri.parse('$baseUrl/realms/$realmName$path');

    if (queryParams != null && queryParams.isNotEmpty) {
      url = url.replace(queryParameters: queryParams);
    }

    final response = await http.get(url, headers: headers);

    if (response.statusCode >= 200 && response.statusCode < 300) {
      return jsonDecode(response.body) as Map<String, dynamic>;
    } else {
      throw ApiException(
        statusCode: response.statusCode,
        message: response.body,
      );
    }
  }

  /// 发送 PUT 请求
  Future<Map<String, dynamic>> put(
    String path,
    Map<String, dynamic> body,
  ) async {
    final url = Uri.parse('$baseUrl/realms/$realmName$path');
    final response = await http.put(
      url,
      headers: headers,
      body: jsonEncode(body),
    );

    if (response.statusCode >= 200 && response.statusCode < 300) {
      return jsonDecode(response.body) as Map<String, dynamic>;
    } else {
      throw ApiException(
        statusCode: response.statusCode,
        message: response.body,
      );
    }
  }

  /// 发送 DELETE 请求
  Future<void> delete(String path) async {
    final url = Uri.parse('$baseUrl/realms/$realmName$path');
    final response = await http.delete(url, headers: headers);

    if (response.statusCode < 200 || response.statusCode >= 300) {
      throw ApiException(
        statusCode: response.statusCode,
        message: response.body,
      );
    }
  }
}

class ApiException implements Exception {
  final int statusCode;
  final String message;

  ApiException({required this.statusCode, required this.message});

  @override
  String toString() => 'ApiException: $statusCode - $message';
}
```

## API 接口使用示例

### 1. 文本分析食物

```dart
/// 分析食物文本
Future<Map<String, dynamic>> analyzeFoodText({
  required String prompt,
  required String input,
}) async {
  final response = await apiClient.post(
    '/food-analysis/text',
    {
      'prompt': prompt,
      'input': input,
    },
  );
  return response;
}

// 使用示例（新设备用户，无需 Authorization 头）
final result = await analyzeFoodText(
  prompt: '分析这个食物的成分和风险',
  input: 'Caesar Salad with croutons and parmesan cheese',
);
// 系统会自动使用 X-Device-Id 创建匿名用户并处理请求
```

### 2. 图片分析食物

```dart
import 'dart:io';
import 'package:http/http.dart' as http;
import 'package:http_parser/http_parser.dart';

/// 分析食物图片
Future<Map<String, dynamic>> analyzeFoodImage({
  required File imageFile,
  String? prompt,
}) async {
  final url = Uri.parse(
    '${apiClient.baseUrl}/realms/${apiClient.realmName}/food-analysis/image',
  );

  final request = http.MultipartRequest('POST', url);

  // 添加 headers
  request.headers.addAll(apiClient.headers);
  request.headers.remove('Content-Type'); // Multipart 会自动设置

  // 添加图片文件
  final imageBytes = await imageFile.readAsBytes();
  request.files.add(
    http.MultipartFile.fromBytes(
      'image',
      imageBytes,
      filename: 'food_image.jpg',
      contentType: MediaType('image', 'jpeg'),
    ),
  );

  // 添加 prompt（如果提供）
  if (prompt != null) {
    request.fields['prompt'] = prompt;
  }

  final streamedResponse = await request.send();
  final response = await http.Response.fromStream(streamedResponse);

  if (response.statusCode >= 200 && response.statusCode < 300) {
    return jsonDecode(response.body) as Map<String, dynamic>;
  } else {
    throw ApiException(
      statusCode: response.statusCode,
      message: response.body,
    );
  }
}

// 使用示例
final imageFile = File('/path/to/food_image.jpg');
final result = await analyzeFoodImage(
  imageFile: imageFile,
  prompt: '分析这个食物的成分和风险',
);
```

### 3. 获取分析历史

```dart
/// 获取分析历史（支持过滤、排序、分页）
Future<Map<String, dynamic>> getAnalysisHistory({
  int? offset,
  int? limit,
  String? promptId,
  String? inputType,
  DateTime? createdAfter,
  DateTime? createdBefore,
  String? sort,
}) async {
  final queryParams = <String, String>{};

  if (offset != null) queryParams['offset'] = offset.toString();
  if (limit != null) queryParams['limit'] = limit.toString();
  if (promptId != null) queryParams['filter[prompt_id]'] = promptId;
  if (inputType != null) queryParams['filter[input_type]'] = inputType;
  if (createdAfter != null) {
    queryParams['filter[created_at][gte]'] = createdAfter.toIso8601String();
  }
  if (createdBefore != null) {
    queryParams['filter[created_at][lte]'] = createdBefore.toIso8601String();
  }
  if (sort != null) queryParams['sort'] = sort;

  return await apiClient.get('/food-analysis', queryParams: queryParams);
}

// 使用示例
final history = await getAnalysisHistory(
  offset: 0,
  limit: 20,
  sort: '-created_at', // 按创建时间倒序
);
```

### 4. 创建反应记录

```dart
/// 创建食物反应记录
Future<Map<String, dynamic>> createFoodReaction({
  String? analysisItemId, // 可选：关联的分析项 ID
  required DateTime eatenAt,
  required String feeling, // 'GREAT' | 'OKAY' | 'MILD_ISSUES' | 'BAD'
  required String symptomOnset, // 'LT_1H' | 'H1_3H' | 'H3_6H' | 'NEXT_DAY'
  required List<String> symptoms, // ['BLOATING', 'PAIN', 'GAS', ...]
  String? notes,
}) async {
  return await apiClient.post(
    '/food-reactions',
    {
      if (analysisItemId != null) 'analysis_item_id': analysisItemId,
      'eaten_at': eatenAt.toIso8601String(),
      'feeling': feeling,
      'symptom_onset': symptomOnset,
      'symptoms': symptoms,
      if (notes != null) 'notes': notes,
    },
  );
}

// 使用示例
final reaction = await createFoodReaction(
  analysisItemId: 'uuid-of-item', // 可选
  eatenAt: DateTime.now(),
  feeling: 'MILD_ISSUES',
  symptomOnset: 'H1_3H',
  symptoms: ['BLOATING', 'GAS'],
  notes: 'Slight bloating after eating',
);
```

### 5. 获取反应记录列表

```dart
/// 获取反应记录列表
Future<Map<String, dynamic>> getFoodReactions({
  int? offset,
  int? limit,
  String? feeling,
  List<String>? feelings,
  String? analysisItemId,
  String? symptomOnset,
  DateTime? eatenAfter,
  DateTime? eatenBefore,
  bool? hasSymptoms,
  String? sort,
}) async {
  final queryParams = <String, String>{};

  if (offset != null) queryParams['offset'] = offset.toString();
  if (limit != null) queryParams['limit'] = limit.toString();
  if (feeling != null) queryParams['filter[feeling]'] = feeling;
  if (feelings != null && feelings.isNotEmpty) {
    queryParams['filter[feeling][in]'] = feelings.join(',');
  }
  if (analysisItemId != null) {
    queryParams['filter[analysis_item_id]'] = analysisItemId;
  }
  if (symptomOnset != null) {
    queryParams['filter[symptom_onset]'] = symptomOnset;
  }
  if (eatenAfter != null) {
    queryParams['filter[eaten_at][gte]'] = eatenAfter.toIso8601String();
  }
  if (eatenBefore != null) {
    queryParams['filter[eaten_at][lte]'] = eatenBefore.toIso8601String();
  }
  if (hasSymptoms != null) {
    queryParams['filter[has_symptoms]'] = hasSymptoms.toString();
  }
  if (sort != null) queryParams['sort'] = sort;

  return await apiClient.get('/food-reactions', queryParams: queryParams);
}

// 使用示例
final reactions = await getFoodReactions(
  offset: 0,
  limit: 20,
  feeling: 'BAD', // 只获取负面反应
  sort: '-eaten_at', // 按进食时间倒序
);
```

### 6. 获取统计概览

```dart
/// 获取个人触发统计概览
Future<Map<String, dynamic>> getStatsOverview() async {
  return await apiClient.get('/food-stats/overview');
}

// 使用示例
final stats = await getStatsOverview();
print('准确度: ${stats['accuracy_level']}%');
print('跟踪反应数: ${stats['tracked_reactions']}');
print('触发食物数: ${stats['triggered_foods']}');
```

### 7. 获取症状统计

```dart
/// 获取症状统计
Future<Map<String, dynamic>> getSymptomStats({
  DateTime? startDate,
  DateTime? endDate,
  String? symptomCode,
  List<String>? symptomCodes,
  String? sort,
}) async {
  final queryParams = <String, String>{};

  if (startDate != null) {
    queryParams['filter[start_date]'] = startDate.toIso8601String();
  }
  if (endDate != null) {
    queryParams['filter[end_date]'] = endDate.toIso8601String();
  }
  if (symptomCode != null) {
    queryParams['filter[symptom_code]'] = symptomCode;
  }
  if (symptomCodes != null && symptomCodes.isNotEmpty) {
    queryParams['filter[symptom_code][in]'] = symptomCodes.join(',');
  }
  if (sort != null) queryParams['sort'] = sort;

  return await apiClient.get('/food-stats/symptoms', queryParams: queryParams);
}
```

### 8. 获取时间序列统计

```dart
/// 获取时间序列统计
Future<Map<String, dynamic>> getTimelineStats({
  required DateTime startDate,
  required DateTime endDate,
  String? granularity, // 'day' | 'week' | 'month'
  List<String>? feelings,
  String? sort,
}) async {
  final queryParams = <String, String>{
    'filter[start_date]': startDate.toIso8601String(),
    'filter[end_date]': endDate.toIso8601String(),
  };

  if (granularity != null) {
    queryParams['filter[granularity]'] = granularity;
  }
  if (feelings != null && feelings.isNotEmpty) {
    queryParams['filter[feeling][in]'] = feelings.join(',');
  }
  if (sort != null) queryParams['sort'] = sort;

  return await apiClient.get('/food-stats/timeline', queryParams: queryParams);
}

// 使用示例
final timeline = await getTimelineStats(
  startDate: DateTime.now().subtract(Duration(days: 30)),
  endDate: DateTime.now(),
  granularity: 'day',
);
```

## 完整示例：Flutter 应用初始化

```dart
import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'device_id_service.dart';
import 'food_scanner_api_client.dart';

class FoodScannerApp extends StatefulWidget {
  @override
  _FoodScannerAppState createState() => _FoodScannerAppState();
}

class _FoodScannerAppState extends State<FoodScannerApp> {
  late FoodScannerApiClient apiClient;
  bool isInitialized = false;

  @override
  void initState() {
    super.initState();
    _initializeApiClient();
  }

  Future<void> _initializeApiClient() async {
    // 1. 获取设备 ID
    final deviceId = await DeviceIdService.getDeviceId();

    // 2. 从配置或环境变量获取 API 基础 URL 和 realm 名称
    const baseUrl = 'https://api.example.com'; // 替换为实际 API 地址
    const realmName = 'default'; // 替换为实际 realm 名称

    // 3. 创建 API 客户端
    apiClient = FoodScannerApiClient(
      baseUrl: baseUrl,
      realmName: realmName,
      deviceId: deviceId,
      // accessToken: null, // 新设备用户不需要 token
    );

    setState(() {
      isInitialized = true;
    });
  }

  @override
  Widget build(BuildContext context) {
    if (!isInitialized) {
      return Scaffold(
        body: Center(child: CircularProgressIndicator()),
      );
    }

    return MaterialApp(
      title: 'Food Scanner',
      home: HomeScreen(apiClient: apiClient),
    );
  }
}
```

## 错误处理

```dart
class ApiErrorHandler {
  static String getErrorMessage(dynamic error) {
    if (error is ApiException) {
      switch (error.statusCode) {
        case 400:
          return '请求参数错误';
        case 401:
          return '未授权，请检查认证信息';
        case 403:
          return '禁止访问';
        case 404:
          return '资源未找到';
        case 500:
          return '服务器内部错误';
        default:
          return '请求失败: ${error.message}';
      }
    }
    return '未知错误: $error';
  }

  static void showError(BuildContext context, dynamic error) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(getErrorMessage(error)),
        backgroundColor: Colors.red,
      ),
    );
  }
}

// 使用示例
try {
  final result = await analyzeFoodText(
    prompt: '分析食物',
    input: 'Caesar Salad',
  );
  // 处理成功结果
} catch (e) {
  ApiErrorHandler.showError(context, e);
}
```

## 数据模型定义

```dart
// 分析结果模型
class FoodAnalysisResult {
  final String id;
  final String requestId;
  final List<DishAnalysis> dishes;
  final DateTime createdAt;

  FoodAnalysisResult({
    required this.id,
    required this.requestId,
    required this.dishes,
    required this.createdAt,
  });

  factory FoodAnalysisResult.fromJson(Map<String, dynamic> json) {
    return FoodAnalysisResult(
      id: json['id'],
      requestId: json['request_id'],
      dishes: (json['dishes'] as List)
          .map((d) => DishAnalysis.fromJson(d))
          .toList(),
      createdAt: DateTime.parse(json['created_at']),
    );
  }
}

class DishAnalysis {
  final String dishName;
  final String safetyLevel; // 'SAFE' | 'CAUTION' | 'UNSAFE'
  final int riskScore;
  final String riskBand; // 'SAFE' | 'MODERATE' | 'HIGH'
  final List<TriggerIngredient> triggers;

  DishAnalysis({
    required this.dishName,
    required this.safetyLevel,
    required this.riskScore,
    required this.riskBand,
    required this.triggers,
  });

  factory DishAnalysis.fromJson(Map<String, dynamic> json) {
    return DishAnalysis(
      dishName: json['dish_name'],
      safetyLevel: json['safety_level'],
      riskScore: json['risk_score'],
      riskBand: json['risk_band'],
      triggers: (json['triggers'] as List)
          .map((t) => TriggerIngredient.fromJson(t))
          .toList(),
    );
  }
}

class TriggerIngredient {
  final String ingredientName;
  final String triggerCategory;
  final String riskLevel;
  final String riskReason;

  TriggerIngredient({
    required this.ingredientName,
    required this.triggerCategory,
    required this.riskLevel,
    required this.riskReason,
  });

  factory TriggerIngredient.fromJson(Map<String, dynamic> json) {
    return TriggerIngredient(
      ingredientName: json['ingredient_name'],
      triggerCategory: json['trigger_category'],
      riskLevel: json['risk_level'],
      riskReason: json['risk_reason'],
    );
  }
}

// 反应记录模型
class FoodReaction {
  final String id;
  final String? analysisItemId;
  final DateTime eatenAt;
  final String feeling;
  final String symptomOnset;
  final List<String> symptoms;
  final String? notes;
  final DateTime createdAt;

  FoodReaction({
    required this.id,
    this.analysisItemId,
    required this.eatenAt,
    required this.feeling,
    required this.symptomOnset,
    required this.symptoms,
    this.notes,
    required this.createdAt,
  });

  factory FoodReaction.fromJson(Map<String, dynamic> json) {
    return FoodReaction(
      id: json['id'],
      analysisItemId: json['analysis_item_id'],
      eatenAt: DateTime.parse(json['eaten_at']),
      feeling: json['feeling'],
      symptomOnset: json['symptom_onset'],
      symptoms: List<String>.from(json['symptoms']),
      notes: json['notes'],
      createdAt: DateTime.parse(json['created_at']),
    );
  }
}
```

## 最佳实践

### 1. 设备 ID 管理

- **优先使用硬件 ID**：Android ID 或 iOS identifierForVendor
- **持久化存储**：如果无法获取硬件 ID，使用 SharedPreferences 存储生成的 UUID
- **一致性**：确保同一设备每次启动使用相同的设备 ID

### 2. 错误处理

- 始终使用 try-catch 包装 API 调用
- 根据状态码提供用户友好的错误提示
- 记录错误日志以便调试

### 3. 网络请求

- 使用 `http` 包进行网络请求
- 设置合理的超时时间
- 处理网络连接错误

### 4. 数据缓存

- 对于频繁访问的数据（如统计概览），考虑本地缓存
- 使用 `shared_preferences` 或 `hive` 进行本地存储

### 5. 状态管理

- 使用 Provider、Riverpod 或 Bloc 管理 API 客户端状态
- 在应用启动时初始化 API 客户端

## 完整示例：使用 Provider 管理状态

```dart
// api_provider.dart
import 'package:flutter/foundation.dart';
import 'food_scanner_api_client.dart';
import 'device_id_service.dart';

class ApiProvider extends ChangeNotifier {
  FoodScannerApiClient? _apiClient;
  bool _isInitialized = false;

  FoodScannerApiClient? get apiClient => _apiClient;
  bool get isInitialized => _isInitialized;

  Future<void> initialize({
    required String baseUrl,
    required String realmName,
  }) async {
    final deviceId = await DeviceIdService.getDeviceId();
    _apiClient = FoodScannerApiClient(
      baseUrl: baseUrl,
      realmName: realmName,
      deviceId: deviceId,
    );
    _isInitialized = true;
    notifyListeners();
  }

  void setAccessToken(String? token) {
    _apiClient?.accessToken = token;
    notifyListeners();
  }
}

// main.dart
void main() {
  runApp(
    MultiProvider(
      providers: [
        ChangeNotifierProvider(create: (_) => ApiProvider()),
      ],
      child: MyApp(),
    ),
  );
}
```

## 注意事项

1. **设备认证模式**：
   - **仅需 `X-Device-Id` 头，完全不需要 `Authorization` 头**
   - 系统会自动创建匿名用户和设备配置
   - 适用于新设备用户或未登录用户

2. **设备 ID 格式**：设备 ID 应该是字符串，建议使用 UUID 格式

3. **Realm 名称**：确保使用正确的 realm 名称，通常为 `default` 或配置的值

4. **API 基础 URL**：根据环境（开发/生产）配置不同的 API 地址

5. **时区处理**：所有日期时间使用 ISO 8601 格式，确保时区正确

6. **图片上传**：图片文件大小建议控制在 5MB 以内

7. **认证头说明**：
   - 设备认证：`X-Device-Id`（必需）
   - 标准认证：`Authorization: Bearer <token>`（可选，仅已登录用户需要）

## 支持的接口列表

### 食物分析
- `POST /realms/{realm_name}/food-analysis/text` - 文本分析
- `POST /realms/{realm_name}/food-analysis/image` - 图片分析
- `GET /realms/{realm_name}/food-analysis` - 获取分析历史
- `GET /realms/{realm_name}/food-analysis/requests` - 获取分析请求列表
- `GET /realms/{realm_name}/food-analysis/requests/{request_id}` - 获取单个请求

### 分析项和触发成分
- `GET /realms/{realm_name}/food-analysis/requests/{request_id}/items` - 获取请求的分析项
- `GET /realms/{realm_name}/food-analysis/items/{item_id}` - 获取单个分析项
- `GET /realms/{realm_name}/food-analysis/items` - 获取所有分析项
- `GET /realms/{realm_name}/food-analysis/items/{item_id}/triggers` - 获取触发成分
- `GET /realms/{realm_name}/food-analysis/triggers/categories` - 获取分类统计

### 反应记录
- `POST /realms/{realm_name}/food-reactions` - 创建反应记录
- `GET /realms/{realm_name}/food-reactions` - 获取反应记录列表
- `GET /realms/{realm_name}/food-reactions/{reaction_id}` - 获取单个反应记录
- `PUT /realms/{realm_name}/food-reactions/{reaction_id}` - 更新反应记录
- `DELETE /realms/{realm_name}/food-reactions/{reaction_id}` - 删除反应记录

### 统计
- `GET /realms/{realm_name}/food-stats/overview` - 获取统计概览
- `GET /realms/{realm_name}/food-stats/symptoms` - 获取症状统计
- `GET /realms/{realm_name}/food-stats/timeline` - 获取时间序列统计

### 设备管理
- `GET /realms/{realm_name}/devices/{device_id}` - 获取或创建设备配置

## 更多信息

详细的 API 文档请参考：
- OpenAPI 文档：`https://api.example.com/swagger-ui`（替换为实际地址）
- 设计文档：`specs/03-food-scanner-HTTP-design.md`

## 常见问题

### Q: 新设备用户是否需要先注册或登录？

**A:** 不需要。新设备用户可以直接使用 `X-Device-Id` 头调用 API，**完全不需要 `Authorization` 头**，系统会自动创建匿名用户和设备配置。

### Q: 设备认证模式下，是否仍需要 Authorization 头？

**A:** **完全不需要**。设备认证模式下，可以完全省略 `Authorization` 头，只需提供 `X-Device-Id` 即可。

**请求示例（仅设备认证）**：
```http
POST /realms/default/food-analysis/text
X-Device-Id: ios-uuid-12345
Content-Type: application/json

{
  "prompt": "分析这个食物",
  "input": "Caesar Salad"
}
```

**注意**：
- ✅ **不需要** `Authorization: Bearer <token>` 头
- ✅ **只需要** `X-Device-Id` 头
- ✅ 系统会自动识别设备并创建匿名用户

### Q: 设备 ID 会变化吗？

**A:**
- **Android**: Android ID 在设备恢复出厂设置前不会变化
- **iOS**: identifierForVendor 在应用卸载重装前不会变化
- **备用方案**: 如果无法获取硬件 ID，使用本地存储的 UUID，确保应用卸载前保持一致

### Q: 如何从匿名用户升级为正式用户？

**A:** 当用户登录后，将 `access_token` 设置到 `FoodScannerApiClient` 的 `accessToken` 属性即可。系统会自动关联设备到登录用户。

```dart
// 用户登录后
apiClient.accessToken = loginResponse['access_token'];
```

### Q: 如何处理网络错误？

**A:** 建议实现重试机制和错误提示：

```dart
Future<T> retryApiCall<T>(
  Future<T> Function() apiCall, {
  int maxRetries = 3,
  Duration delay = const Duration(seconds: 1),
}) async {
  int attempts = 0;
  while (attempts < maxRetries) {
    try {
      return await apiCall();
    } catch (e) {
      attempts++;
      if (attempts >= maxRetries) rethrow;
      await Future.delayed(delay * attempts);
    }
  }
  throw Exception('Max retries exceeded');
}

// 使用示例
try {
  final result = await retryApiCall(() => analyzeFoodText(
    prompt: '分析食物',
    input: 'Caesar Salad',
  ));
} catch (e) {
  // 处理错误
}
```

### Q: 图片上传大小限制是多少？

**A:** 建议图片大小控制在 5MB 以内，格式支持 JPEG、PNG。如果图片过大，建议在客户端先进行压缩：

```dart
import 'package:image/image.dart' as img;
import 'dart:io';

Future<File> compressImage(File imageFile, {int maxSizeKB = 500}) async {
  final imageBytes = await imageFile.readAsBytes();
  var image = img.decodeImage(imageBytes);

  if (image == null) throw Exception('Failed to decode image');

  // 计算压缩比例
  final currentSizeKB = imageBytes.length / 1024;
  if (currentSizeKB <= maxSizeKB) return imageFile;

  var quality = (maxSizeKB / currentSizeKB * 100).round().clamp(10, 100);

  // 压缩图片
  final compressedBytes = img.encodeJpg(image, quality: quality);

  // 保存到临时文件
  final tempFile = File('${imageFile.path}_compressed.jpg');
  await tempFile.writeAsBytes(compressedBytes);

  return tempFile;
}
```

### Q: 如何实现离线缓存？

**A:** 可以使用 `hive` 或 `shared_preferences` 缓存 API 响应：

```dart
import 'package:hive/hive.dart';

class CacheService {
  static late Box _cacheBox;

  static Future<void> init() async {
    await Hive.initFlutter();
    _cacheBox = await Hive.openBox('api_cache');
  }

  static Future<T?> get<T>(String key) async {
    final cached = _cacheBox.get(key);
    if (cached != null) {
      final data = Map<String, dynamic>.from(cached);
      final timestamp = DateTime.parse(data['timestamp']);
      // 缓存有效期 5 分钟
      if (DateTime.now().difference(timestamp).inMinutes < 5) {
        return data['data'] as T;
      }
    }
    return null;
  }

  static Future<void> set(String key, dynamic value) async {
    await _cacheBox.put(key, {
      'data': value,
      'timestamp': DateTime.now().toIso8601String(),
    });
  }
}
```

### Q: 如何测试 API 调用？

**A:** 可以使用 `http` 包的 mock 或使用 `mockito` 进行单元测试：

```dart
import 'package:http/http.dart' as http;
import 'package:mockito/mockito.dart';
import 'package:mockito/annotations.dart';

@GenerateMocks([http.Client])
void main() {
  test('analyzeFoodText should return result', () async {
    final mockClient = MockClient();
    when(mockClient.post(any, headers: anyNamed('headers'), body: anyNamed('body')))
        .thenAnswer((_) async => http.Response(
          '{"id": "test-id", "request_id": "test-request-id"}',
          200,
        ));

    // 测试代码
  });
}
```
