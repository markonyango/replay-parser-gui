import { themeQuartz } from 'ag-grid-community';

/** Grim-dark grid theme shared by the match list and the match details dialog. */
export const gridTheme = themeQuartz.withParams({
  browserColorScheme: 'dark',
  backgroundColor: '#1b1714',
  foregroundColor: '#e3d8bd',
  headerBackgroundColor: '#100d0b',
  headerTextColor: '#c0993f',
  headerFontFamily: "'Cinzel', serif",
  headerFontSize: 13,
  headerFontWeight: 700,
  fontFamily: "'Barlow', sans-serif",
  oddRowBackgroundColor: '#201b17',
  rowHoverColor: '#3a2a1c',
  selectedRowBackgroundColor: '#4a1f1c',
  borderColor: '#4a3f30',
  accentColor: '#c0993f',
  borderRadius: 2,
  wrapperBorderRadius: 2,
  inputBackgroundColor: '#100d0b',
});
