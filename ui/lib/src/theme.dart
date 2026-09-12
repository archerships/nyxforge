import 'package:flutter/material.dart';

// NyxForge colour palette -- dark, minimal, privacy-first aesthetic
class NyxColors {
  NyxColors._();

  static const background   = Color(0xFF0D0F1A);
  static const surface      = Color(0xFF1E2030);
  static const surfaceHigh  = Color(0xFF2A2D3E);
  static const border       = Color(0xFF2A2A3D);

  static const primary      = Color(0xFF5E6AD2); // bright accent purple
  static const secondary    = Color(0xFF2A2D3E);
  static const accent       = Color(0xFF5E6AD2);
  static const accentBright = Color(0xFF7C89F8);
  static const accentGlow   = Color(0x335E6AD2);

  static const textPrimary   = Color(0xFFFFFFFF);
  static const textSecondary = Color(0xFFA0A8D0);
  static const textMuted     = Color(0xFF50505E);

  static const success = Color(0xFF4ADE80);
  static const warning = Color(0xFFFBBF24);
  static const danger  = Color(0xFFF87171);
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
