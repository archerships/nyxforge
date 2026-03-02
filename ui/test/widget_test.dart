import 'package:flutter_test/flutter_test.dart';
import 'package:nyxforge_ui/main.dart';

void main() {
  testWidgets('Splash screen renders app name', (WidgetTester tester) async {
    await tester.pumpWidget(const NyxForgeApp());
    await tester.pump(); // allow initState animations to start

    expect(find.text('NYXFORGE'), findsOneWidget);
  });
}
