import type { FunctionComponent } from 'react'
import Link from 'next/link'
import AppBar from '@mui/material/AppBar'
import Button from '@mui/material/Button'
import IconButton from '@mui/material/IconButton'
import Stack from '@mui/material/Stack'
import Toolbar from '@mui/material/Toolbar'
import Typography from '@mui/material/Typography'
import LibraryAddOutlinedIcon from '@mui/icons-material/LibraryAddOutlined'

import styles from './styles.module.scss'

const Header: FunctionComponent = () => (
  <AppBar>
    <Toolbar>
      <Typography variant="h6" noWrap component="div">
        Hoarder
      </Typography>
      <Stack className={styles.nav} spacing={1} flexGrow={1} direction="row">
        <Link href="/" passHref legacyBehavior>
          <Button color="inherit">
            ホーム
          </Button>
        </Link>
        <Link href="/tags" passHref legacyBehavior>
          <Button color="inherit">
            タグ
          </Button>
        </Link>
        <Link href="/sources" passHref legacyBehavior>
          <Button color="inherit">
            サービス
          </Button>
        </Link>
      </Stack>
      <Stack>
        <Link href="/media/new" passHref legacyBehavior>
          <IconButton size="large" color="inherit">
            <LibraryAddOutlinedIcon />
          </IconButton>
        </Link>
      </Stack>
    </Toolbar>
  </AppBar>
)

export default Header
