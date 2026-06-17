import 'package:flutter/material.dart';

/// EDITORS-PRO Design System v2
///
/// Professional dark theme inspired by:
/// - DaVinci Resolve (color grading workflow)
/// - CapCut Pro (mobile-first dark mode)
/// - Adobe Premiere Pro (timeline + panel layout)
class AppTheme {
  AppTheme._();

  // ─── Brand Colors ──────────────────────────────────────────────
  /// Primary brand purple — used for CTAs, active states, gradients
  static const Color primary = Color(0xFF6C5CE7);

  /// Lighter purple — for hovers, secondary accents, gradients
  static const Color primaryLight = Color(0xFFA29BFE);

  /// Deep purple — for pressed states, borders
  static const Color primaryDark = Color(0xFF4834D4);

  /// Teal — secondary accent for timeline/audio elements
  static const Color secondary = Color(0xFF00CEC9);

  /// Pink — used for export button, accent highlights
  static const Color accent = Color(0xFFFD79A8);

  // ─── Surface Tones (Layered Dark) ──────────────────────────────
  /// App background — deepest layer
  static const Color background = Color(0xFF08080D);

  /// Primary surface — main panels, toolbars
  static const Color surface = Color(0xFF12121B);

  /// Elevated surface — cards, popovers
  static const Color surfaceVariant = Color(0xFF1A1A26);

  /// Card background — slightly elevated
  static const Color cardColor = Color(0xFF1E1E2E);

  /// Border / divider color
  static const Color border = Color(0xFF2A2A3E);
  static const Color borderLight = Color(0xFF353550);

  // ─── Text Tones ────────────────────────────────────────────────
  static const Color textPrimary = Color(0xFFF0F0F8);
  static const Color textSecondary = Color(0xFF9E9EB8);
  static const Color textDisabled = Color(0xFF4A4A62);

  // ─── Status Colors ─────────────────────────────────────────────
  static const Color success = Color(0xFF00D9A0);
  static const Color warning = Color(0xFFFFB84D);
  static const Color error = Color(0xFFFF5C5C);
  static const Color info = Color(0xFF4DA6FF);

  // ─── Track Colors (Timeline) ───────────────────────────────────
  static const Color videoTrackColor = Color(0xFF8B7FE8);  // Soft purple
  static const Color videoTrackColorLight = Color(0xFFB5AAFF);
  static const Color audioTrackColor = Color(0xFF00D9A0);  // Bright green
  static const Color audioTrackColorLight = Color(0xFF5FE5C7);
  static const Color textTrackColor = Color(0xFFFFB84D);   // Amber
  static const Color effectTrackColor = Color(0xFFFF79C6); // Hot pink
  static const Color playheadColor = Color(0xFFFF3B5C);    // Crimson

  // ─── Gradients ─────────────────────────────────────────────────
  static const LinearGradient primaryGradient = LinearGradient(
    begin: Alignment.topLeft,
    end: Alignment.bottomRight,
    colors: [Color(0xFF6C5CE7), Color(0xFFA29BFE)],
  );

  static const LinearGradient secondaryGradient = LinearGradient(
    begin: Alignment.topLeft,
    end: Alignment.bottomRight,
    colors: [Color(0xFF00CEC9), Color(0xFF55EFC4)],
  );

  static const LinearGradient accentGradient = LinearGradient(
    begin: Alignment.topLeft,
    end: Alignment.bottomRight,
    colors: [Color(0xFFFD79A8), Color(0xFFFF9FAB)],
  );

  static const LinearGradient sunsetGradient = LinearGradient(
    begin: Alignment.topLeft,
    end: Alignment.bottomRight,
    colors: [Color(0xFF6C5CE7), Color(0xFF00CEC9), Color(0xFFFD79A8)],
  );

  /// Background gradient for splash/onboarding
  static const LinearGradient backgroundGradient = LinearGradient(
    begin: Alignment.topCenter,
    end: Alignment.bottomCenter,
    colors: [Color(0xFF08080D), Color(0xFF12121B), Color(0xFF08080D)],
  );

  // ─── Shadows & Glows ───────────────────────────────────────────
  static List<BoxShadow> primaryGlow({double opacity = 0.4}) => [
    BoxShadow(
      color: primary.withOpacity(opacity),
      blurRadius: 24,
      offset: const Offset(0, 4),
    ),
  ];

  static List<BoxShadow> accentGlow({double opacity = 0.3}) => [
    BoxShadow(
      color: accent.withOpacity(opacity),
      blurRadius: 16,
      offset: const Offset(0, 2),
    ),
  ];

  static List<BoxShadow> softShadow = [
    BoxShadow(
      color: Colors.black.withOpacity(0.3),
      blurRadius: 12,
      offset: const Offset(0, 4),
    ),
  ];

  // ─── Spacing Scale (4-pt grid) ─────────────────────────────────
  static const double spacing4 = 4.0;
  static const double spacing8 = 8.0;
  static const double spacing12 = 12.0;
  static const double spacing16 = 16.0;
  static const double spacing20 = 20.0;
  static const double spacing24 = 24.0;
  static const double spacing32 = 32.0;
  static const double spacing48 = 48.0;

  // ─── Border Radius ─────────────────────────────────────────────
  static const double radiusSmall = 6.0;
  static const double radiusMedium = 10.0;
  static const double radiusLarge = 16.0;
  static const double radiusXLarge = 24.0;
  static const double radiusFull = 999.0;

  // ─── Layout constants ──────────────────────────────────────────
  static const double timelineMinHeight = 220.0;
  static const double trackHeight = 56.0;
  static const double clipMinWidth = 24.0;
  static const double playheadWidth = 2.0;

  // ─── Light Theme Tones (Phase E.5) ──────────────────────────────
  // A light variant for outdoor / bright-environment editing. Brand
  // colors (primary, secondary, accent) are shared with the dark theme
  // for consistency; only the surface tones and text colors change.
  static const Color lightBackground = Color(0xFFF6F6FA);
  static const Color lightSurface = Color(0xFFFFFFFF);
  static const Color lightSurfaceVariant = Color(0xFFEEEEF4);
  static const Color lightCardColor = Color(0xFFFFFFFF);
  static const Color lightBorder = Color(0xFFD8D8E2);
  static const Color lightBorderLight = Color(0xFFC0C0CE);
  static const Color lightTextPrimary = Color(0xFF1A1A26);
  static const Color lightTextSecondary = Color(0xFF5A5A72);
  static const Color lightTextDisabled = Color(0xFFB0B0BE);

  static final ThemeData darkTheme = ThemeData(
    useMaterial3: true,
    brightness: Brightness.dark,
    colorSchemeSeed: primary,
    scaffoldBackgroundColor: background,
    fontFamily: 'Inter',

    appBarTheme: const AppBarTheme(
      backgroundColor: surface,
      foregroundColor: textPrimary,
      elevation: 0,
      centerTitle: false,
      titleTextStyle: TextStyle(
        fontFamily: 'Inter',
        fontSize: 20,
        fontWeight: FontWeight.w700,
        color: textPrimary,
      ),
    ),

    cardTheme: CardTheme(
      color: cardColor,
      elevation: 0,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(radiusLarge),
      ),
      margin: const EdgeInsets.symmetric(horizontal: spacing16, vertical: spacing8),
    ),

    elevatedButtonTheme: ElevatedButtonThemeData(
      style: ElevatedButton.styleFrom(
        backgroundColor: primary,
        foregroundColor: Colors.white,
        elevation: 0,
        padding: const EdgeInsets.symmetric(horizontal: spacing24, vertical: spacing12),
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(radiusMedium),
        ),
        textStyle: const TextStyle(
          fontFamily: 'Inter',
          fontSize: 14,
          fontWeight: FontWeight.w600,
        ),
      ),
    ),

    outlinedButtonTheme: OutlinedButtonThemeData(
      style: OutlinedButton.styleFrom(
        foregroundColor: primary,
        side: const BorderSide(color: primary, width: 1.5),
        padding: const EdgeInsets.symmetric(horizontal: spacing24, vertical: spacing12),
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(radiusMedium),
        ),
        textStyle: const TextStyle(
          fontFamily: 'Inter',
          fontSize: 14,
          fontWeight: FontWeight.w600,
        ),
      ),
    ),

    textButtonTheme: TextButtonThemeData(
      style: TextButton.styleFrom(
        foregroundColor: primaryLight,
        textStyle: const TextStyle(
          fontFamily: 'Inter',
          fontSize: 14,
          fontWeight: FontWeight.w500,
        ),
      ),
    ),

    iconTheme: const IconThemeData(
      color: textSecondary,
      size: 24,
    ),

    bottomNavigationBarTheme: const BottomNavigationBarThemeData(
      backgroundColor: surface,
      selectedItemColor: primary,
      unselectedItemColor: textDisabled,
      type: BottomNavigationBarType.fixed,
      elevation: 8,
    ),

    dividerTheme: const DividerThemeData(
      color: border,
      thickness: 1,
    ),

    inputDecorationTheme: InputDecorationTheme(
      filled: true,
      fillColor: surfaceVariant,
      border: OutlineInputBorder(
        borderRadius: BorderRadius.circular(radiusMedium),
        borderSide: BorderSide.none,
      ),
      focusedBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(radiusMedium),
        borderSide: const BorderSide(color: primary, width: 1.5),
      ),
      hintStyle: const TextStyle(color: textDisabled),
      contentPadding: const EdgeInsets.symmetric(horizontal: spacing16, vertical: spacing12),
    ),

    sliderTheme: SliderThemeData(
      activeTrackColor: primary,
      thumbColor: primaryLight,
      inactiveTrackColor: border,
      trackHeight: 3,
      thumbShape: const RoundSliderThumbShape(enabledThumbRadius: 7),
    ),

    snackBarTheme: SnackBarThemeData(
      backgroundColor: surfaceVariant,
      contentTextStyle: const TextStyle(color: textPrimary),
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(radiusMedium),
      ),
      behavior: SnackBarBehavior.floating,
    ),

    dialogTheme: DialogTheme(
      backgroundColor: surface,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(radiusLarge),
      ),
    ),

    bottomSheetTheme: const BottomSheetThemeData(
      backgroundColor: surface,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(radiusXLarge)),
      ),
    ),

    chipTheme: ChipThemeData(
      backgroundColor: surfaceVariant,
      selectedColor: primary.withOpacity(0.2),
      labelStyle: const TextStyle(color: textPrimary, fontSize: 12),
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(radiusSmall),
      ),
      side: BorderSide.none,
    ),

    tabBarTheme: TabBarTheme(
      labelColor: primary,
      unselectedLabelColor: textSecondary,
      indicatorColor: primary,
      indicatorSize: TabBarIndicatorSize.label,
      labelStyle: const TextStyle(fontWeight: FontWeight.w600, fontSize: 14),
      unselectedLabelStyle: const TextStyle(fontWeight: FontWeight.w400, fontSize: 14),
    ),

    textTheme: const TextTheme(
      headlineLarge: TextStyle(fontSize: 28, fontWeight: FontWeight.w700, color: textPrimary, letterSpacing: -0.5),
      headlineMedium: TextStyle(fontSize: 24, fontWeight: FontWeight.w700, color: textPrimary),
      headlineSmall: TextStyle(fontSize: 20, fontWeight: FontWeight.w600, color: textPrimary),
      titleLarge: TextStyle(fontSize: 18, fontWeight: FontWeight.w600, color: textPrimary),
      titleMedium: TextStyle(fontSize: 16, fontWeight: FontWeight.w600, color: textPrimary),
      titleSmall: TextStyle(fontSize: 14, fontWeight: FontWeight.w600, color: textPrimary),
      bodyLarge: TextStyle(fontSize: 16, fontWeight: FontWeight.w400, color: textPrimary),
      bodyMedium: TextStyle(fontSize: 14, fontWeight: FontWeight.w400, color: textPrimary),
      bodySmall: TextStyle(fontSize: 12, fontWeight: FontWeight.w400, color: textSecondary),
      labelLarge: TextStyle(fontSize: 14, fontWeight: FontWeight.w600, color: textPrimary),
      labelMedium: TextStyle(fontSize: 12, fontWeight: FontWeight.w500, color: textSecondary),
      labelSmall: TextStyle(fontSize: 10, fontWeight: FontWeight.w500, color: textDisabled),
    ),
  );

  /// Phase E.5: Light theme variant.
  ///
  /// Shares brand colors (primary, secondary, accent) with [darkTheme]
  /// for consistency. Only surface tones, borders, and text colors are
  /// swapped to light equivalents. Use this when the user has selected
  /// "Light" in Settings > Appearance, or when the system is in light
  /// mode and the user has chosen "Follow system".
  static final ThemeData lightTheme = ThemeData(
    useMaterial3: true,
    brightness: Brightness.light,
    colorSchemeSeed: primary,
    scaffoldBackgroundColor: lightBackground,
    fontFamily: 'Inter',

    appBarTheme: const AppBarTheme(
      backgroundColor: lightSurface,
      foregroundColor: lightTextPrimary,
      elevation: 0,
      centerTitle: false,
      titleTextStyle: TextStyle(
        fontFamily: 'Inter',
        fontSize: 20,
        fontWeight: FontWeight.w700,
        color: lightTextPrimary,
      ),
    ),

    cardTheme: CardTheme(
      color: lightCardColor,
      elevation: 0,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(radiusLarge),
      ),
      margin: const EdgeInsets.symmetric(horizontal: spacing16, vertical: spacing8),
    ),

    elevatedButtonTheme: ElevatedButtonThemeData(
      style: ElevatedButton.styleFrom(
        backgroundColor: primary,
        foregroundColor: Colors.white,
        elevation: 0,
        padding: const EdgeInsets.symmetric(horizontal: spacing24, vertical: spacing12),
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(radiusMedium),
        ),
        textStyle: const TextStyle(
          fontFamily: 'Inter',
          fontSize: 14,
          fontWeight: FontWeight.w600,
        ),
      ),
    ),

    outlinedButtonTheme: OutlinedButtonThemeData(
      style: OutlinedButton.styleFrom(
        foregroundColor: primary,
        side: const BorderSide(color: primary, width: 1.5),
        padding: const EdgeInsets.symmetric(horizontal: spacing24, vertical: spacing12),
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(radiusMedium),
        ),
        textStyle: const TextStyle(
          fontFamily: 'Inter',
          fontSize: 14,
          fontWeight: FontWeight.w600,
        ),
      ),
    ),

    textButtonTheme: TextButtonThemeData(
      style: TextButton.styleFrom(
        foregroundColor: primaryDark,
        textStyle: const TextStyle(
          fontFamily: 'Inter',
          fontSize: 14,
          fontWeight: FontWeight.w500,
        ),
      ),
    ),

    iconTheme: const IconThemeData(
      color: lightTextSecondary,
      size: 24,
    ),

    bottomNavigationBarTheme: const BottomNavigationBarThemeData(
      backgroundColor: lightSurface,
      selectedItemColor: primary,
      unselectedItemColor: lightTextDisabled,
      type: BottomNavigationBarType.fixed,
      elevation: 8,
    ),

    dividerTheme: const DividerThemeData(
      color: lightBorder,
      thickness: 1,
    ),

    inputDecorationTheme: InputDecorationTheme(
      filled: true,
      fillColor: lightSurfaceVariant,
      border: OutlineInputBorder(
        borderRadius: BorderRadius.circular(radiusMedium),
        borderSide: BorderSide.none,
      ),
      focusedBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(radiusMedium),
        borderSide: const BorderSide(color: primary, width: 1.5),
      ),
      hintStyle: const TextStyle(color: lightTextDisabled),
      contentPadding: const EdgeInsets.symmetric(horizontal: spacing16, vertical: spacing12),
    ),

    sliderTheme: SliderThemeData(
      activeTrackColor: primary,
      thumbColor: primaryLight,
      inactiveTrackColor: lightBorder,
      trackHeight: 3,
      thumbShape: const RoundSliderThumbShape(enabledThumbRadius: 7),
    ),

    snackBarTheme: SnackBarThemeData(
      backgroundColor: lightSurfaceVariant,
      contentTextStyle: const TextStyle(color: lightTextPrimary),
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(radiusMedium),
      ),
      behavior: SnackBarBehavior.floating,
    ),

    dialogTheme: DialogTheme(
      backgroundColor: lightSurface,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(radiusLarge),
      ),
    ),

    bottomSheetTheme: const BottomSheetThemeData(
      backgroundColor: lightSurface,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(radiusXLarge)),
      ),
    ),

    chipTheme: ChipThemeData(
      backgroundColor: lightSurfaceVariant,
      selectedColor: primary.withOpacity(0.2),
      labelStyle: const TextStyle(color: lightTextPrimary, fontSize: 12),
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(radiusSmall),
      ),
      side: BorderSide.none,
    ),

    tabBarTheme: TabBarTheme(
      labelColor: primary,
      unselectedLabelColor: lightTextSecondary,
      indicatorColor: primary,
      indicatorSize: TabBarIndicatorSize.label,
      labelStyle: const TextStyle(fontWeight: FontWeight.w600, fontSize: 14),
      unselectedLabelStyle: const TextStyle(fontWeight: FontWeight.w400, fontSize: 14),
    ),

    textTheme: const TextTheme(
      headlineLarge: TextStyle(fontSize: 28, fontWeight: FontWeight.w700, color: lightTextPrimary, letterSpacing: -0.5),
      headlineMedium: TextStyle(fontSize: 24, fontWeight: FontWeight.w700, color: lightTextPrimary),
      headlineSmall: TextStyle(fontSize: 20, fontWeight: FontWeight.w600, color: lightTextPrimary),
      titleLarge: TextStyle(fontSize: 18, fontWeight: FontWeight.w600, color: lightTextPrimary),
      titleMedium: TextStyle(fontSize: 16, fontWeight: FontWeight.w600, color: lightTextPrimary),
      titleSmall: TextStyle(fontSize: 14, fontWeight: FontWeight.w600, color: lightTextPrimary),
      bodyLarge: TextStyle(fontSize: 16, fontWeight: FontWeight.w400, color: lightTextPrimary),
      bodyMedium: TextStyle(fontSize: 14, fontWeight: FontWeight.w400, color: lightTextPrimary),
      bodySmall: TextStyle(fontSize: 12, fontWeight: FontWeight.w400, color: lightTextSecondary),
      labelLarge: TextStyle(fontSize: 14, fontWeight: FontWeight.w600, color: lightTextPrimary),
      labelMedium: TextStyle(fontSize: 12, fontWeight: FontWeight.w500, color: lightTextSecondary),
      labelSmall: TextStyle(fontSize: 10, fontWeight: FontWeight.w500, color: lightTextDisabled),
    ),
  );
}
