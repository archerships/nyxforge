import 'package:flutter/material.dart';

// NyxForge colour palette -- dark, minimal, privacy-first aesthetic
class NyxColors {
  NyxColors._();

  static const background   = Color(0xFF0A0A0F);
  static const surface      = Color(0xFF12121A);
  static const surfaceHigh  = Color(0xFF1C1C28);
  static const border       = Color(0xFF2A2A3D);

  static const accent       = Color(0xFF7B5EA7); // muted purple -- DarkFi nod
  static const accentBright = Color(0xFFAA82D9);
  static const accentGlow   = Color(0x337B5EA7);

  static const textPrimary   = Color(0xFFE8E8F0);
  static const textSecondary = Color(0xFF9090A8);
  static const textMuted     = Color(0xFF50505E);

  static const success = Color(0xFF4CAF82);
  static const warning = Color(0xFFD4A843);
  static const danger  = Color(0xFFCF5858);
}

ThemeData nyxTheme() {
  return ThemeData(
    useMaterial3: true,
    brightness: Brightness.dark,
    scaffoldBackgroundColor: NyxColors.background,
    colorScheme: const ColorScheme.dark(
      surface:          NyxColors.surface,
      primary:          NyxColors.accent,
      onPrimary:        NyxColors.textPrimary,
      secondary:        NyxColors.accentBright,
      onSecondary:      NyxColors.background,
      onSurface:        NyxColors.textPrimary,
      outline:          NyxColors.border,
    ),
    textTheme: const TextTheme(
      displaySmall: TextStyle(
        color: NyxColors.textPrimary,
        fontSize: 28,
        fontWeight: FontWeight.w300,
        letterSpacing: 2,
      ),
      titleLarge: TextStyle(
        color: NyxColors.textPrimary,
        fontSize: 18,
        fontWeight: FontWeight.w500,
        letterSpacing: 0.5,
      ),
      bodyMedium: TextStyle(color: NyxColors.textSecondary, fontSize: 14),
      bodySmall:  TextStyle(color: NyxColors.textMuted,     fontSize: 12),
      labelLarge: TextStyle(
        color: NyxColors.accentBright,
        fontSize: 13,
        fontWeight: FontWeight.w600,
        letterSpacing: 1.2,
      ),
    ),
    appBarTheme: const AppBarTheme(
      backgroundColor:  NyxColors.background,
      foregroundColor:  NyxColors.textPrimary,
      elevation:        0,
      centerTitle:      false,
      titleTextStyle: TextStyle(
        color:       NyxColors.textPrimary,
        fontSize:    18,
        fontWeight:  FontWeight.w300,
        letterSpacing: 3,
      ),
    ),
    navigationRailTheme: const NavigationRailThemeData(
      backgroundColor:          NyxColors.surface,
      indicatorColor:           NyxColors.accentGlow,
      selectedIconTheme:        IconThemeData(color: NyxColors.accentBright),
      unselectedIconTheme:      IconThemeData(color: NyxColors.textMuted),
      selectedLabelTextStyle:   TextStyle(color: NyxColors.accentBright, fontSize: 11),
      unselectedLabelTextStyle: TextStyle(color: NyxColors.textMuted,    fontSize: 11),
    ),
    cardTheme: CardThemeData(
      color:     NyxColors.surfaceHigh,
      elevation: 0,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(8),
        side: const BorderSide(color: NyxColors.border, width: 1),
      ),
    ),
    dividerTheme: const DividerThemeData(color: NyxColors.border, thickness: 1),
    elevatedButtonTheme: ElevatedButtonThemeData(
      style: ElevatedButton.styleFrom(
        backgroundColor: NyxColors.accent,
        foregroundColor: NyxColors.textPrimary,
        elevation:       0,
        padding:  const EdgeInsets.symmetric(horizontal: 24, vertical: 14),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(6)),
        textStyle: const TextStyle(letterSpacing: 1, fontWeight: FontWeight.w600),
      ),
    ),
    outlinedButtonTheme: OutlinedButtonThemeData(
      style: OutlinedButton.styleFrom(
        foregroundColor: NyxColors.accentBright,
        side:   const BorderSide(color: NyxColors.border),
        padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 12),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(6)),
      ),
    ),
    inputDecorationTheme: const InputDecorationTheme(
      filled:      true,
      fillColor:   NyxColors.surface,
      border: OutlineInputBorder(
        borderSide: BorderSide(color: NyxColors.border),
      ),
      enabledBorder: OutlineInputBorder(
        borderSide: BorderSide(color: NyxColors.border),
      ),
      focusedBorder: OutlineInputBorder(
        borderSide: BorderSide(color: NyxColors.accent, width: 2),
      ),
      labelStyle: TextStyle(color: NyxColors.textSecondary),
      hintStyle:  TextStyle(color: NyxColors.textMuted),
    ),
    chipTheme: ChipThemeData(
      backgroundColor: NyxColors.surfaceHigh,
      labelStyle: const TextStyle(color: NyxColors.textSecondary, fontSize: 12),
      side:  const BorderSide(color: NyxColors.border),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(4)),
    ),
  );
}
