/// Ferriskey 移动端认证 SDK (Dart/Flutter)
///
/// 支持两种认证流程：
/// 1. Password Flow (推荐) - 直接使用用户名密码获取令牌
/// 2. Authorization Code Flow - 完整的 OAuth 2.0 流程

import 'dart:convert';
import 'package:http/http.dart' as http;
import 'package:flutter_secure_storage/flutter_secure_storage.dart';

class FerriskeyAuthClient {
  final String baseUrl;
  final String realmName;
  final String clientId;
  final String? clientSecret;
  final FlutterSecureStorage _storage = const FlutterSecureStorage();

  FerriskeyAuthClient({
    required this.baseUrl,
    required this.realmName,
    required this.clientId,
    this.clientSecret,
  });

  String get _tokenEndpoint =>
      '$baseUrl/realms/$realmName/protocol/openid-connect/token';

  String get _authEndpoint =>
      '$baseUrl/realms/$realmName/protocol/openid-connect/auth';

  String get _authenticateEndpoint =>
      '$baseUrl/realms/$realmName/login-actions/authenticate';

  String get _registrationEndpoint =>
      '$baseUrl/realms/$realmName/protocol/openid-connect/registrations';

  // ============================================================
  // 方式 1: Password Flow (推荐用于移动 App)
  // ============================================================

  /// 使用用户名密码直接登录获取令牌
  ///
  /// 这是最简单的方式，适合移动 App
  /// 需要服务端 client 启用 direct_access_grants_enabled
  Future<TokenResponse> loginWithPassword({
    required String username,
    required String password,
  }) async {
    try {
      final response = await http.post(
        Uri.parse(_tokenEndpoint),
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
        },
        body: {
          'grant_type': 'password',
          'client_id': clientId,
          if (clientSecret != null) 'client_secret': clientSecret!,
          'username': username,
          'password': password,
        },
      );

      if (response.statusCode == 200) {
        final tokenResponse = TokenResponse.fromJson(
          json.decode(response.body),
        );

        // 保存令牌到安全存储
        await _saveTokens(tokenResponse);

        return tokenResponse;
      } else {
        throw AuthException(
          'Login failed: ${response.statusCode}',
          response.body,
        );
      }
    } catch (e) {
      throw AuthException('Login error', e.toString());
    }
  }

  // ============================================================
  // 方式 2: Authorization Code Flow (完整 OAuth 2.0 流程)
  // ============================================================

  /// 步骤 1: 初始化认证流程，获取登录 URL
  ///
  /// 这会返回一个登录页面 URL 和 session cookies
  /// 适合需要在 WebView 中展示自定义登录页面的场景
  Future<AuthInitResponse> initializeAuth({
    required String redirectUri,
    String? state,
    String? scope,
  }) async {
    final queryParams = {
      'response_type': 'code',
      'client_id': clientId,
      'redirect_uri': redirectUri,
      if (scope != null) 'scope': scope,
      if (state != null) 'state': state,
    };

    final uri = Uri.parse(_authEndpoint).replace(
      queryParameters: queryParams,
    );

    try {
      final response = await http.get(
        uri,
        headers: {
          'Accept': 'application/json',
        },
      );

      if (response.statusCode == 302 || response.statusCode == 200) {
        final data = json.decode(response.body);

        // 保存 session cookies
        final cookies = response.headers['set-cookie'];
        if (cookies != null) {
          await _storage.write(key: 'auth_cookies', value: cookies);
        }

        return AuthInitResponse(
          loginUrl: data['url'],
          sessionCookies: cookies,
        );
      } else {
        throw AuthException(
          'Auth initialization failed: ${response.statusCode}',
          response.body,
        );
      }
    } catch (e) {
      throw AuthException('Auth initialization error', e.toString());
    }
  }

  /// 步骤 2: 使用用户名密码进行认证
  ///
  /// 需要携带之前保存的 session cookies
  Future<AuthenticateResponse> authenticate({
    required String username,
    required String password,
  }) async {
    final cookies = await _storage.read(key: 'auth_cookies');
    if (cookies == null) {
      throw AuthException(
        'No session cookies found',
        'Please call initializeAuth() first',
      );
    }

    final uri = Uri.parse(_authenticateEndpoint).replace(
      queryParameters: {'client_id': clientId},
    );

    try {
      final response = await http.post(
        uri,
        headers: {
          'Content-Type': 'application/json',
          'Cookie': cookies,
        },
        body: json.encode({
          'username': username,
          'password': password,
        }),
      );

      if (response.statusCode == 200) {
        final data = json.decode(response.body);
        return AuthenticateResponse.fromJson(data);
      } else {
        throw AuthException(
          'Authentication failed: ${response.statusCode}',
          response.body,
        );
      }
    } catch (e) {
      throw AuthException('Authentication error', e.toString());
    }
  }

  /// 步骤 3: 使用授权码交换令牌
  ///
  /// 在认证成功后，使用返回的授权码获取访问令牌
  Future<TokenResponse> exchangeCodeForToken({
    required String code,
  }) async {
    try {
      final response = await http.post(
        Uri.parse(_tokenEndpoint),
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
        },
        body: {
          'grant_type': 'authorization_code',
          'client_id': clientId,
          if (clientSecret != null) 'client_secret': clientSecret!,
          'code': code,
        },
      );

      if (response.statusCode == 200) {
        final tokenResponse = TokenResponse.fromJson(
          json.decode(response.body),
        );

        await _saveTokens(tokenResponse);

        return tokenResponse;
      } else {
        throw AuthException(
          'Token exchange failed: ${response.statusCode}',
          response.body,
        );
      }
    } catch (e) {
      throw AuthException('Token exchange error', e.toString());
    }
  }

  // ============================================================
  // 令牌管理
  // ============================================================

  /// 刷新访问令牌
  Future<TokenResponse> refreshToken() async {
    final refreshToken = await _storage.read(key: 'refresh_token');
    if (refreshToken == null) {
      throw AuthException('No refresh token found', 'Please login again');
    }

    try {
      final response = await http.post(
        Uri.parse(_tokenEndpoint),
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
        },
        body: {
          'grant_type': 'refresh_token',
          'client_id': clientId,
          if (clientSecret != null) 'client_secret': clientSecret!,
          'refresh_token': refreshToken,
        },
      );

      if (response.statusCode == 200) {
        final tokenResponse = TokenResponse.fromJson(
          json.decode(response.body),
        );

        await _saveTokens(tokenResponse);

        return tokenResponse;
      } else {
        throw AuthException(
          'Token refresh failed: ${response.statusCode}',
          response.body,
        );
      }
    } catch (e) {
      throw AuthException('Token refresh error', e.toString());
    }
  }

  /// 获取当前访问令牌
  Future<String?> getAccessToken() async {
    return await _storage.read(key: 'access_token');
  }

  /// 获取当前刷新令牌
  Future<String?> getRefreshToken() async {
    return await _storage.read(key: 'refresh_token');
  }

  /// 检查令牌是否已过期
  Future<bool> isTokenExpired() async {
    final expiresAtStr = await _storage.read(key: 'token_expires_at');
    if (expiresAtStr == null) return true;

    final expiresAt = DateTime.parse(expiresAtStr);
    return DateTime.now().isAfter(expiresAt);
  }

  /// 自动刷新令牌（如果过期）
  Future<String> getValidAccessToken() async {
    if (await isTokenExpired()) {
      final tokenResponse = await refreshToken();
      return tokenResponse.accessToken;
    }

    final token = await getAccessToken();
    if (token == null) {
      throw AuthException('No access token', 'Please login first');
    }

    return token;
  }

  /// 登出
  Future<void> logout() async {
    await _storage.delete(key: 'access_token');
    await _storage.delete(key: 'refresh_token');
    await _storage.delete(key: 'id_token');
    await _storage.delete(key: 'token_expires_at');
    await _storage.delete(key: 'auth_cookies');
  }

  // ============================================================
  // 用户注册
  // ============================================================

  /// 注册新用户
  ///
  /// 注意：需要 realm 启用用户注册功能
  Future<TokenResponse> register({
    required String username,
    required String email,
    required String password,
    String? firstName,
    String? lastName,
  }) async {
    try {
      final response = await http.post(
        Uri.parse(_registrationEndpoint),
        headers: {
          'Content-Type': 'application/json',
        },
        body: json.encode({
          'username': username,
          'email': email,
          'password': password,
          if (firstName != null) 'first_name': firstName,
          if (lastName != null) 'last_name': lastName,
        }),
      );

      if (response.statusCode == 201) {
        final tokenResponse = TokenResponse.fromJson(
          json.decode(response.body),
        );

        await _saveTokens(tokenResponse);

        return tokenResponse;
      } else {
        throw AuthException(
          'Registration failed: ${response.statusCode}',
          response.body,
        );
      }
    } catch (e) {
      throw AuthException('Registration error', e.toString());
    }
  }

  // ============================================================
  // 私有辅助方法
  // ============================================================

  Future<void> _saveTokens(TokenResponse tokenResponse) async {
    await _storage.write(key: 'access_token', value: tokenResponse.accessToken);
    await _storage.write(key: 'refresh_token', value: tokenResponse.refreshToken);
    await _storage.write(key: 'id_token', value: tokenResponse.idToken);

    // 计算过期时间（提前 30 秒）
    final expiresAt = DateTime.now().add(
      Duration(seconds: tokenResponse.expiresIn - 30),
    );
    await _storage.write(key: 'token_expires_at', value: expiresAt.toIso8601String());
  }
}

// ============================================================
// 数据模型
// ============================================================

class TokenResponse {
  final String accessToken;
  final String tokenType;
  final String refreshToken;
  final int expiresIn;
  final String idToken;

  TokenResponse({
    required this.accessToken,
    required this.tokenType,
    required this.refreshToken,
    required this.expiresIn,
    required this.idToken,
  });

  factory TokenResponse.fromJson(Map<String, dynamic> json) {
    return TokenResponse(
      accessToken: json['access_token'],
      tokenType: json['token_type'],
      refreshToken: json['refresh_token'],
      expiresIn: json['expires_in'],
      idToken: json['id_token'],
    );
  }
}

class AuthInitResponse {
  final String loginUrl;
  final String? sessionCookies;

  AuthInitResponse({
    required this.loginUrl,
    this.sessionCookies,
  });
}

class AuthenticateResponse {
  final AuthenticationStatus status;
  final String? url;
  final List<String>? requiredActions;
  final String? token;
  final String? message;

  AuthenticateResponse({
    required this.status,
    this.url,
    this.requiredActions,
    this.token,
    this.message,
  });

  factory AuthenticateResponse.fromJson(Map<String, dynamic> json) {
    return AuthenticateResponse(
      status: AuthenticationStatus.fromString(json['status']),
      url: json['url'],
      requiredActions: json['required_actions'] != null
          ? List<String>.from(json['required_actions'])
          : null,
      token: json['token'],
      message: json['message'],
    );
  }

  /// 从重定向 URL 中提取授权码
  String? extractCodeFromUrl() {
    if (url == null) return null;

    final uri = Uri.parse(url!);
    return uri.queryParameters['code'];
  }
}

enum AuthenticationStatus {
  success,
  requiresActions,
  requiresOtpChallenge,
  failed;

  static AuthenticationStatus fromString(String status) {
    switch (status.toLowerCase()) {
      case 'success':
        return AuthenticationStatus.success;
      case 'requiresactions':
        return AuthenticationStatus.requiresActions;
      case 'requiresotpchallenge':
        return AuthenticationStatus.requiresOtpChallenge;
      case 'failed':
        return AuthenticationStatus.failed;
      default:
        throw ArgumentError('Unknown authentication status: $status');
    }
  }
}

class AuthException implements Exception {
  final String message;
  final String details;

  AuthException(this.message, this.details);

  @override
  String toString() => 'AuthException: $message\nDetails: $details';
}

// ============================================================
// HTTP 客户端 (带自动令牌刷新)
// ============================================================

class AuthenticatedHttpClient {
  final FerriskeyAuthClient authClient;
  final http.Client _httpClient = http.Client();

  AuthenticatedHttpClient(this.authClient);

  /// 发送带认证的 GET 请求
  Future<http.Response> get(
    String url, {
    Map<String, String>? headers,
  }) async {
    final token = await authClient.getValidAccessToken();

    return _httpClient.get(
      Uri.parse(url),
      headers: {
        'Authorization': 'Bearer $token',
        ...?headers,
      },
    );
  }

  /// 发送带认证的 POST 请求
  Future<http.Response> post(
    String url, {
    Map<String, String>? headers,
    Object? body,
  }) async {
    final token = await authClient.getValidAccessToken();

    return _httpClient.post(
      Uri.parse(url),
      headers: {
        'Authorization': 'Bearer $token',
        'Content-Type': 'application/json',
        ...?headers,
      },
      body: body,
    );
  }

  /// 发送带认证的 PUT 请求
  Future<http.Response> put(
    String url, {
    Map<String, String>? headers,
    Object? body,
  }) async {
    final token = await authClient.getValidAccessToken();

    return _httpClient.put(
      Uri.parse(url),
      headers: {
        'Authorization': 'Bearer $token',
        'Content-Type': 'application/json',
        ...?headers,
      },
      body: body,
    );
  }

  /// 发送带认证的 DELETE 请求
  Future<http.Response> delete(
    String url, {
    Map<String, String>? headers,
  }) async {
    final token = await authClient.getValidAccessToken();

    return _httpClient.delete(
      Uri.parse(url),
      headers: {
        'Authorization': 'Bearer $token',
        ...?headers,
      },
    );
  }

  void close() {
    _httpClient.close();
  }
}

// ============================================================
// 使用示例
// ============================================================

void main() async {
  // 初始化认证客户端
  final authClient = FerriskeyAuthClient(
    baseUrl: 'https://your-ferriskey-server.com',
    realmName: 'your-realm',
    clientId: 'your-mobile-app-client-id',
    // clientSecret: 'your-client-secret', // 可选，公共客户端不需要
  );

  // ============================================================
  // 方式 1: Password Flow (推荐)
  // ============================================================

  try {
    // 登录
    final tokenResponse = await authClient.loginWithPassword(
      username: 'user@example.com',
      password: 'password123',
    );

    print('Login successful!');
    print('Access Token: ${tokenResponse.accessToken}');
    print('Expires in: ${tokenResponse.expiresIn} seconds');

    // 创建认证 HTTP 客户端
    final httpClient = AuthenticatedHttpClient(authClient);

    // 发送带认证的请求
    final response = await httpClient.get(
      'https://your-ferriskey-server.com/api/users/me',
    );

    print('User info: ${response.body}');

    // 登出
    await authClient.logout();

  } on AuthException catch (e) {
    print('Authentication error: $e');
  }

  // ============================================================
  // 方式 2: Authorization Code Flow
  // ============================================================

  try {
    // 步骤 1: 初始化认证
    final authInit = await authClient.initializeAuth(
      redirectUri: 'myapp://callback',
      state: 'random-state-string',
      scope: 'openid profile email',
    );

    print('Login URL: ${authInit.loginUrl}');

    // 步骤 2: 用户输入凭证进行认证
    final authResult = await authClient.authenticate(
      username: 'user@example.com',
      password: 'password123',
    );

    if (authResult.status == AuthenticationStatus.success) {
      // 步骤 3: 从重定向 URL 提取授权码
      final code = authResult.extractCodeFromUrl();

      if (code != null) {
        // 步骤 4: 使用授权码交换令牌
        final tokenResponse = await authClient.exchangeCodeForToken(
          code: code,
        );

        print('Login successful!');
        print('Access Token: ${tokenResponse.accessToken}');
      }
    } else if (authResult.status == AuthenticationStatus.requiresActions) {
      print('Required actions: ${authResult.requiredActions}');
      // 处理所需操作（如邮箱验证、OTP 设置等）
    } else if (authResult.status == AuthenticationStatus.failed) {
      print('Authentication failed: ${authResult.message}');
    }

  } on AuthException catch (e) {
    print('Authentication error: $e');
  }

  // ============================================================
  // 用户注册
  // ============================================================

  try {
    final tokenResponse = await authClient.register(
      username: 'newuser',
      email: 'newuser@example.com',
      password: 'securePassword123',
      firstName: 'John',
      lastName: 'Doe',
    );

    print('Registration successful!');
    print('Access Token: ${tokenResponse.accessToken}');

  } on AuthException catch (e) {
    print('Registration error: $e');
  }

  // ============================================================
  // 令牌刷新示例
  // ============================================================

  try {
    // 检查令牌是否过期
    if (await authClient.isTokenExpired()) {
      print('Token expired, refreshing...');
      await authClient.refreshToken();
      print('Token refreshed!');
    }

    // 或者直接获取有效令牌（自动刷新）
    final validToken = await authClient.getValidAccessToken();
    print('Valid token: $validToken');

  } on AuthException catch (e) {
    print('Token refresh error: $e');
  }
}
