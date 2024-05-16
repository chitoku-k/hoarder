import { createTheme } from '@mui/material'
import cyan from '@mui/material/colors/cyan'
import { jaJP } from '@mui/material/locale'

const theme = createTheme(
  {
    palette: {
      mode: 'light',
      primary: {
        main: cyan[900],
      },
    },
  },
  jaJP,
)

export default theme
