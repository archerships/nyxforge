import 'package:flutter/material.dart';
import 'src/theme.dart';
import 'src/home_screen.dart';

void main() {
  runApp(const NyxForgeApp());
}

class NyxForgeApp extends StatelessWidget {
  const NyxForgeApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title:        'NyxForge',
      debugShowCheckedModeBanner: false,
      theme:        nyxTheme(),
      home:         const _SplashGate(),
    );
  }
}

/// Shows a brief splash, then transitions to the main shell.
class _SplashGate extends StatefulWidget {
  const _SplashGate();

  @override
  State<_SplashGate> createState() => _SplashGateState();
}

class _SplashGateState extends State<_SplashGate>
    with SingleTickerProviderStateMixin {
  late final AnimationController _ctrl;
  late final Animation<double>   _fade;

  @override
  void initState() {
    super.initState();
    _ctrl = AnimationController(
      vsync:    this,
      duration: const Duration(milliseconds: 900),
    );
    _fade = CurvedAnimation(parent: _ctrl, curve: Curves.easeIn);
    _ctrl.forward().then((_) async {
      await Future<void>.delayed(const Duration(milliseconds: 600));
      if (!mounted) return;
      Navigator.of(context).pushReplacement(
        PageRouteBuilder(
          pageBuilder: (context, animation, secondaryAnimation) =>
              const HomeScreen(),
          transitionsBuilder: (context, animation, secondaryAnimation, child) =>
              FadeTransition(opacity: animation, child: child),
          transitionDuration: const Duration(milliseconds: 500),
        ),
      );
    });
  }

  @override
  void dispose() {
    _ctrl.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: NyxColors.background,
      body: Center(
        child: FadeTransition(
          opacity: _fade,
          child: const Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              _Logo(),
              SizedBox(height: 24),
              Text(
                'NYXFORGE',
                style: TextStyle(
                  color:       NyxColors.textPrimary,
                  fontSize:    32,
                  fontWeight:  FontWeight.w200,
                  letterSpacing: 10,
                ),
              ),
              SizedBox(height: 8),
              Text(
                'anonymous social policy bond market',
                style: TextStyle(
                  color:     NyxColors.textMuted,
                  fontSize:  13,
                  letterSpacing: 2,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _Logo extends StatelessWidget {
  const _Logo();

  @override
  Widget build(BuildContext context) {
    return Container(
      width:  72,
      height: 72,
      decoration: BoxDecoration(
        color:        NyxColors.accentGlow,
        borderRadius: BorderRadius.circular(16),
        border:       Border.all(color: NyxColors.accent, width: 2),
        boxShadow: [
          BoxShadow(
            color:       NyxColors.accent.withValues(alpha: 0.3),
            blurRadius:  24,
            spreadRadius: 2,
          ),
        ],
      ),
      child: const Icon(
        Icons.hub,
        color: NyxColors.accentBright,
        size:  38,
      ),
    );
  }
}
